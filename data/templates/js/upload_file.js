const form = document.querySelector("#upload-form");
const formOut = document.querySelector("#formout");
const dropContainer = document.querySelector("#dropContainer");
const fileInput = document.querySelector("#fileInput");
const previewContainer = document.getElementById("preview-container");

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

// Utility function to prevent default browser behavior
function preventDefaults(e) {
  e.preventDefault();
  e.stopPropagation();
}

// Preventing default browser behavior when dragging a file over the container
dropContainer.addEventListener("dragover", preventDefaults);
dropContainer.addEventListener("dragenter", preventDefaults);
dropContainer.addEventListener("dragleave", preventDefaults);

// Handling dropping files into the Container
dropContainer.addEventListener("drop", handleDrop);

fileInput.addEventListener("change", function (something) {
  if (something.target.files[0]) {
    makePreview(something.target.files[0]);
  }
});

function isValidImage(file) {
  const allowedTypes = ["image/jpeg", "image/png", "image/gif"];
  return allowedTypes.includes(file.type);
}

// We’ll discuss `handleDrop` function down the road
function handleDrop(e) {
  e.preventDefault();

  const files = e.dataTransfer.files;

  if (files.length) {
    fileInput.files = files;
  }
}

function makePreview(file) {
  console.log("processing file");
  const reader = new FileReader();
  reader.readAsDataURL(file);

  // Once the file has been loaded, fire the processing
  reader.onloadend = function (e) {
    var preview;
    if (isValidImage(file)) {
      preview = document.createElement("img");
      preview.src = e.target.result;
    } else {
      preview = document.createElement("p");
      preview.textContent = "No preview for this file type";
    }
    preview.id = "preview-actual";
    preview.classList.add("m-5");

    // remove the old stuff if there is any
    var maybe_old = document.getElementById("preview-actual");
    if (maybe_old) {
      previewContainer.removeChild(maybe_old);
    }

    previewContainer.appendChild(preview);
  };
}
