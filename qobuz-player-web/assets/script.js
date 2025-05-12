let lastOpenTime = Date.now();

document.addEventListener("visibilitychange", () => {
  let wakeUp = Date.now() - lastOpenTime > 5000;

  if (!document.hidden && !wakeUp) {
    location.reload();
  }
});

setInterval(() => {
  (lastOpenTime = Date.now()), 1000;
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
