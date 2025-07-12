const events = ["position", "status", "tracklist", "volume"];

let evtSource;

function initSse() {
  evtSource = new EventSource("/sse");

  for (const event of events) {
    evtSource.addEventListener(event, (_event) => {
      handleSse(event);
    });
  }
}

initSse();

document.addEventListener("visibilitychange", () => {
  if (!document.hidden) {
    initSse();
  }
});

function handleSse(query) {
  const elements = document.querySelectorAll(`[data-sse="${query}"]`);

  for (const element of elements) {
    htmx.trigger(element, "sse");
  }
}

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
