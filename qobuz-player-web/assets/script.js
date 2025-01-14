document.addEventListener("visibilitychange", () => {
  if (!document.hidden) {
    location.reload();
  }
});

function focusSearch() {
  document.getElementById("query").focus();
}
