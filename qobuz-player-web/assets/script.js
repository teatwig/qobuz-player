document.addEventListener("visibilitychange", () => {
  if (!document.hidden) {
    location.reload();
  }
});

function focusSearchInput() {
  document.getElementById("query").focus();
}

function loadSearchInput() {
  let value = sessionStorage.getItem("search-query");
  document.getElementById("query").value = value;
}

function setSearchQuery(value) {
  sessionStorage.setItem("search-query", value);
}
