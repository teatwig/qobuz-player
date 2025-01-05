document.addEventListener("visibilitychange", () => {
  if (!document.hidden) {
    // location.reload();
    htmx.ajax("GET", "");
  }
});
