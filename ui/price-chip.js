// Paint the monthly list price independently of the account's usage windows.
(function (root, factory) {
  const api = factory();
  if (typeof module === "object" && module.exports) module.exports = api;
  if (root && root.document) root.QhudPriceChip = api;
})(typeof window === "undefined" ? null : window, function () {
  "use strict";

  const BASE_USD = 200;
  const rendered = new WeakMap();

  function patch(chip, price) {
    const key = price ? JSON.stringify([price.kind, price.min, price.max, price.label, price.detail]) : null;
    if (rendered.get(chip) === key) return;
    rendered.set(chip, key);
    chip.hidden = !price;
    chip.textContent = price?.label || "";
    if (!price) {
      chip.removeAttribute("title");
      chip.removeAttribute("role");
      chip.removeAttribute("aria-label");
      delete chip.dataset.kind;
      delete chip.dataset.aboveBase;
      chip.style.removeProperty("--price-units");
      chip.style.removeProperty("--price-min");
      chip.style.removeProperty("--price-max");
      return;
    }

    chip.title = price.detail;
    chip.setAttribute("role", "img");
    chip.setAttribute("aria-label", price.detail);
    chip.dataset.kind = price.kind;
    // A small price still gets a full-width track for its text. Only the
    // coloured part measures dollars. Above the reference, the track grows.
    const scale = Math.max(BASE_USD, price.max ?? 0);
    chip.toggleAttribute("data-above-base", scale > BASE_USD);
    chip.style.setProperty("--price-units", String(scale / BASE_USD));
    chip.style.setProperty("--price-min", `${100 * (price.min ?? 0) / scale}%`);
    chip.style.setProperty("--price-max", `${100 * (price.max ?? 0) / scale}%`);
  }

  return { patch };
});
