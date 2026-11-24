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
    const key = price ? JSON.stringify([price.kind, price.max, price.label, price.cycle, price.detail]) : null;
    if (rendered.get(chip) === key) return;
    rendered.set(chip, key);
    chip.hidden = !price;
    chip.textContent = price?.label || "";
    if (!price) {
      chip.removeAttribute("title");
      chip.removeAttribute("role");
      chip.removeAttribute("aria-label");
      delete chip.dataset.kind;
      delete chip.dataset.cycle;
      delete chip.dataset.aboveBase;
      chip.style.removeProperty("--price-units");
      return;
    }

    chip.title = price.detail;
    chip.setAttribute("role", "img");
    chip.setAttribute("aria-label", price.detail);
    chip.dataset.kind = price.kind;
    if (price.cycle === "m" || price.cycle === "y") chip.dataset.cycle = price.cycle;
    else delete chip.dataset.cycle;
    const usd = price.max ?? 0;
    chip.toggleAttribute("data-above-base", usd > BASE_USD);
    chip.style.setProperty("--price-units", String(usd / BASE_USD));
  }

  return { patch };
});
