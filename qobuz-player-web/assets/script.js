const events = ["status", "tracklist"];

let evtSource;

function initSse() {
  evtSource = new EventSource("/sse");

  for (const event of events) {
    evtSource.addEventListener(event, (_event) => {
      handleSse(event);
    });
  }

  evtSource.addEventListener("volume", (event) => {
    const slider = document.getElementById("volume-slider");
    slider.value = event.data;
  });

  evtSource.addEventListener("position", (event) => {
    const slider = document.getElementById("progress-slider");
    slider.value = event.data;

    const positionElement = document.getElementById("position");

    const minutes = Math.floor(event.data / 60)
      .toString()
      .padStart(2, "0");
    const seconds = (event.data % 60).toString().padStart(2, "0");

    positionElement.innerText = `${minutes}:${seconds}`;
  });
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
