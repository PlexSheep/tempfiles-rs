const form = document.querySelector("#upload-form");
const formOut = document.querySelector("#formout");

async function sendData() {
  // Associate the FormData object with the form element
  const formData = new FormData(form);

  try {
    const response = await fetch("/api/v1/file", {
      method: "POST",
      // Set the FormData instance as the request body
      body: formData,
    });
    response.json().then((response_stuff) => {
      var url_frontend = response_stuff.url_frontend;
      formOut.innerHTML = `View Upload: <a href=${url_frontend}>: ${url_frontend}</a>`;
      window.location.replace(url_frontend);
    });
  } catch (e) {
    console.error(e);
  }
}
// Take over form submission
form.addEventListener("submit", (event) => {
  event.preventDefault();
  sendData();
});
