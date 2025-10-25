const MODIFIER_KEYS: Record<string, number> = Object.freeze({
  ShiftLeft: 1 << 0,
  ShiftRight: 1 << 1,
  AltLeft: 1 << 2,
  AltRight: 1 << 3,
  ControlLeft: 1 << 4,
  ControlRight: 1 << 5,
  MetaLeft: 1 << 6,
  MetaRight: 1 << 7,
});

export class ModifierKeys {
  value = 0;

  handledEvent(e: KeyboardEvent) {
    const keyBitMask = MODIFIER_KEYS[e.code];
    if (!keyBitMask) return false;
    if (e.type === "keydown") {
      this.value |= keyBitMask;
    } else {
      this.value &= ~keyBitMask;
    }
    return true;
  }
}
