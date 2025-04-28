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
		var fid = response["fid"];
		var raw_uri = response["fid"];
	} catch (e) {
		console.error(e);
	}
}
// Take over form submission
form.addEventListener("submit", (event) => {
	event.preventDefault();
	sendData();
});
