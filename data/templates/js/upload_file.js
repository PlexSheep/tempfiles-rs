const form = document.querySelector("#upload-form");
const formOut = document.querySelector("#formout");
const dropContainer = document.querySelector("#dropContainer");
const fileInput = document.querySelector("#fileInput");
const textInput = document.querySelector("#textInput");
const fileTypeSelect = document.querySelector("#fileTypeSelect");
const customFileName = document.querySelector("#customFileName");
const previewContainer = document.getElementById("preview-container");
const textPreviewContainer = document.getElementById("text-preview-container");
const formStatus = document.getElementById("formstatus");

// Tab elements
const fileTab = document.getElementById("file-tab");
const textTab = document.getElementById("text-tab");
const filePane = document.getElementById("file-pane");
const textPane = document.getElementById("text-pane");

async function sendData() {
	// Create FormData object
	const formData = new FormData();

	// Check which tab is active
	const isTextMode = textPane.classList.contains("show") && textPane.classList.contains("active");

	if (isTextMode) {
		// Handle text paste upload
		const textContent = textInput.value.trim();
		if (!textContent) {
			formStatus.innerHTML = "Please enter some text to upload";
			return;
		}

		// Generate filename
		const fileExtension = fileTypeSelect.value;
		const fileName = customFileName.value.trim()
			? `${customFileName.value}.${fileExtension}`
			: `paste.${fileExtension}`;

		// Create a File object from the text content
		const textFile = new File([textContent], fileName, {
			type: getContentType(fileExtension)
		});

		// Add the file to FormData
		formData.append("file", textFile);
	} else {
		// Handle regular file upload
		const file = fileInput.files[0];
		if (!file) {
			formStatus.innerHTML = "Please select a file to upload";
			return;
		}
		formData.append("file", file);
	}

	console.info("Trying the upload");
	try {
		const response = await fetch("/api/v1/file", {
			method: "POST",
			body: formData,
		});

		if (response.status == 401) {
			formStatus.innerHTML =
				"Not logged in and anonymous uploads are disabled by the administrator";
			return;
		}
		if (!response.ok) {
			throw Error("Could not upload file");
		}

		response.json().then((response_stuff) => {
			var url_frontend = response_stuff.url_frontend;
			if (url_frontend != null) {
				formOut.innerHTML = `View Upload: <a href=${url_frontend}>: ${url_frontend}</a>`;
				window.location.replace(url_frontend);
			}
		});
	} catch (e) {
		console.error(e);
		formStatus.innerHTML = "Error while uploading";
	}
}

// Get appropriate MIME type for file extension
function getContentType(extension) {
	const contentTypes = {
		'txt': 'text/plain',
		'md': 'text/markdown',
		'rs': 'text/x-rust',
		'py': 'text/x-python',
		'js': 'text/javascript',
		'css': 'text/css',
		'html': 'text/html',
		'json': 'application/json',
		'xml': 'application/xml',
		'yaml': 'application/x-yaml',
		'toml': 'application/toml',
		'sh': 'text/x-shellscript',
		'sql': 'application/sql',
		'log': 'text/plain'
	};
	return contentTypes[extension] || 'text/plain';
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

// File input change handler
fileInput.addEventListener("change", function(something) {
	if (something.target.files[0]) {
		makePreview(something.target.files[0]);
	}
});

textInput.addEventListener("input", makeTextPreview);
fileTypeSelect.addEventListener("change", makeTextPreview);
customFileName.addEventListener("input", makeTextPreview);

function isValidImage(file) {
	const allowedTypes = ["image/jpeg", "image/png", "image/gif"];
	return allowedTypes.includes(file.type);
}

// Handle drag and drop for files
function handleDrop(e) {
	e.preventDefault();

	const files = e.dataTransfer.files;

	if (files.length) {
		// Switch to file tab if dropping files
		const fileTabTrigger = new bootstrap.Tab(fileTab);
		fileTabTrigger.show();

		fileInput.files = files;
		makePreview(files[0]);
	}
}

// Make preview for uploaded files
function makePreview(file) {
	console.log("processing file");
	const reader = new FileReader();
	reader.readAsDataURL(file);

	reader.onloadend = function(e) {
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

		// Remove the old preview if there is any
		var maybe_old = document.getElementById("preview-actual");
		if (maybe_old) {
			previewContainer.removeChild(maybe_old);
		}

		previewContainer.appendChild(preview);
	};
}

// Make preview for text content
function makeTextPreview() {
	const textContent = textInput.value.trim();

	// Remove old preview
	const oldPreview = document.getElementById("text-preview-actual");
	if (oldPreview) {
		textPreviewContainer.removeChild(oldPreview);
	}

	if (textContent) {
		const preview = document.createElement("div");
		preview.id = "text-preview-actual";
		preview.classList.add("mt-3", "p-3", "bg-secondary-subtle", "border", "rounded");

		const previewHeader = document.createElement("small");
		previewHeader.classList.add("text-muted");
		const fileExtension = fileTypeSelect.value;
		const fileName = customFileName.value.trim()
			? `${customFileName.value}.${fileExtension}`
			: `paste.${fileExtension}`;
		previewHeader.textContent = `Preview: ${fileName} (${textContent.length} characters)`;

		const previewContent = document.createElement("pre");
		previewContent.classList.add("mt-2", "mb-0", "font-monospace", "small");
		previewContent.style.maxHeight = "200px";
		previewContent.style.overflow = "auto";
		previewContent.textContent = textContent.length > 500
			? textContent.substring(0, 500) + "\n... (truncated)"
			: textContent;

		preview.appendChild(previewHeader);
		preview.appendChild(previewContent);
		textPreviewContainer.appendChild(preview);
	}
}

// Tab switching handlers
document.addEventListener('DOMContentLoaded', function() {
	const tabs = document.querySelectorAll('#uploadTabs button[data-bs-toggle="tab"]');
	tabs.forEach(tab => {
		tab.addEventListener('shown.bs.tab', function(event) {
			// Clear form status when switching tabs
			formStatus.innerHTML = "";

			// Clear previews when switching tabs
			const oldFilePreview = document.getElementById("preview-actual");
			if (oldFilePreview) {
				previewContainer.removeChild(oldFilePreview);
			}

			const oldTextPreview = document.getElementById("text-preview-actual");
			if (oldTextPreview) {
				textPreviewContainer.removeChild(oldTextPreview);
			}
		});
	});
});
