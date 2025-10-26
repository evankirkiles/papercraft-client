import styles from "./styles.module.scss";

export default function SidePanel() {
  return (
    <section className={styles.container} aria-label="Side Panel">
      <button className={styles.header}>DEVELOPER TOOLS</button>
      <div className={styles.content}>
        <h2>Controls (Trackpad-oriented):</h2>
        <ul>
          <li>Scroll: Orbit</li>
          <li>CMD + Scroll: Zoom</li>
          <li>SHIFT + Scroll: Pan</li>
          <li>TAB: Toggle X-Ray Mode</li>
          <li>S / Alt+S: Mark cut line</li>
          <li>D: Switch tab edge</li>
          <li>G: Move selected piece (in 2D viewport)</li>
          <li>CTRL + Scroll: Tween between folded / unfolded state</li>
        </ul>
        <hr />
        <h2>Log</h2>
        <h3>06/15/2025</h3>
        <p>Set up this site to share progress as I work on the tool.</p>
        <p>
          At this point, most of the basics are there. It supports reading
          GLTFs, processing their meshes into a BMesh (the adjacency structure
          used by Blender), managing and caching GPU resources to draw the model
          on the `{"<canvas />"}` with `wgpu`, marking edges as "cut lines" in
          the mesh and automatically creating pieces for valid cut-out sections,
          GPU-based object picking / selection in four modes (vert, edge, face,
          piece), a configurable line and vertex dot thickness, and providing a
          very basic 2D layout mode for arranging pieces.
        </p>
        <p>
          I've ignored some performance improvements trying to figure out if
          this is 100% possible. I'm definitely not caching GPU resources as
          well as I could and many operations require walking a large part of
          the mesh, but this is probably as bad performance-wise as it's gonna
          get, so that's good at least. Papercraftable models are generally very
          low in polycounts, so there's a lot less pressure.
        </p>
        <p>
          A major performance upgrade will come by default with WebGPU browser
          compatibility, as this is currently running on WebGL with a
          WebGPU-like interface (`wgpu`). WWDC25 seems to imply that Safari will
          finally catch up, so that might happen sooner than expected.
        </p>
      </div>
    </section>
  );
}
