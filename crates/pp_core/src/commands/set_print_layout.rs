use serde::{Deserialize, Serialize};

use crate::print::PrintLayoutSettings;

use super::{Command, CommandError};

/// Changes the page size and / or margins of the document's print layout.
///
/// Neither direction is analytically invertible - a page size is overwritten,
/// not nudged - so the command carries both the previous and the next settings
/// as snapshots, in the same spirit as [`super::update_flaps::UpdateFlapsCommand`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetPrintLayoutCommand {
    pub before: PrintLayoutSettings,
    pub after: PrintLayoutSettings,
}

impl Command for SetPrintLayoutCommand {
    fn execute(&self, state: &mut crate::State) -> Result<(), CommandError> {
        state.printing.apply_settings(&self.after);
        Ok(())
    }

    fn rollback(&self, state: &mut crate::State) -> Result<(), CommandError> {
        state.printing.apply_settings(&self.before);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use cgmath::Vector3;

    use super::*;
    use crate::{
        bounds::Aabb3,
        measures::Dimensions,
        print::{PageSize, CM_PER_INCH},
        State,
    };

    fn command(after: PrintLayoutSettings, state: &State) -> SetPrintLayoutCommand {
        SetPrintLayoutCommand { before: state.printing.settings(), after }
    }

    /// The round trip every undoable command owes: executing then rolling back
    /// leaves the layout exactly where it started.
    #[test]
    fn execute_and_rollback_round_trip() {
        let mut state = State::default();
        let original = state.printing.settings();
        assert_eq!(original.page_size, PageSize::A4);

        let cmd = command(
            PrintLayoutSettings { page_size: PageSize::Letter, margin_x: 1.0, margin_y: 2.0 },
            &state,
        );
        cmd.execute(&mut state).unwrap();
        assert_eq!(state.printing.settings().page_size, PageSize::Letter);
        assert_eq!(state.printing.page_margin_start.x, 1.0);
        // Margins are symmetric: the end corner tracks the start corner
        assert_eq!(state.printing.page_margin_end.y, 2.0);

        cmd.rollback(&mut state).unwrap();
        assert_eq!(state.printing.settings(), original);
        assert_eq!(state.printing.page_margin_start.x, 0.5 * CM_PER_INCH);
        assert_eq!(state.printing.page_margin_end.x, 0.5 * CM_PER_INCH);
    }

    /// A smaller page needs more sheets to cover the same pieces, so the grid
    /// has to refit after a size change - otherwise the pieces would hang off
    /// the layout until something else happened to trigger a fit.
    #[test]
    fn the_page_grid_refits_after_a_size_change() {
        let mut state = State::default();
        // Pieces spanning exactly one A4 sheet
        let Dimensions { width, height } = PageSize::A4.dimensions();
        let mut bounds = Aabb3::EMPTY;
        bounds.extend(Vector3::new(0.0, 0.0, 0.0));
        bounds.extend(Vector3::new(width, -height, 0.0));
        state.printing.fit_to_bounds(&bounds);
        assert_eq!((state.printing.cols, state.printing.rows), (1, 1));

        // Halving the page in both directions should take four sheets to cover
        let half = Dimensions { width: width / 2.0, height: height / 2.0 };
        let cmd = command(
            PrintLayoutSettings { page_size: PageSize::Custom(half), margin_x: 0.0, margin_y: 0.0 },
            &state,
        );
        cmd.execute(&mut state).unwrap();
        state.printing.fit_to_bounds(&bounds);
        assert_eq!((state.printing.cols, state.printing.rows), (2, 2));

        cmd.rollback(&mut state).unwrap();
        state.printing.fit_to_bounds(&bounds);
        assert_eq!((state.printing.cols, state.printing.rows), (1, 1));
    }

    /// The renderer only re-uploads the print uniform when the layout is dirty,
    /// so both directions have to flag it.
    #[test]
    fn both_directions_mark_the_layout_dirty() {
        let mut state = State::default();
        let cmd = command(
            PrintLayoutSettings { page_size: PageSize::Letter, margin_x: 0.0, margin_y: 0.0 },
            &state,
        );

        state.printing.is_dirty = false;
        cmd.execute(&mut state).unwrap();
        assert!(state.printing.is_dirty);

        state.printing.is_dirty = false;
        cmd.rollback(&mut state).unwrap();
        assert!(state.printing.is_dirty);
    }
}
