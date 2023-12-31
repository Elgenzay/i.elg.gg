document.addEventListener('DOMContentLoaded', (e) => {
	document.body.addEventListener('paste', function (e) {
		const items = (event.clipboardData || event.originalEvent.clipboardData).items;

		for (const item of items) {
			if (item.kind === 'file') {
				const file = item.getAsFile();
				uploadFile(file);
				break;
			}
		}
	});

	document.getElementById('uploadButton').addEventListener('click', function () {
		const fileInput = document.getElementById('file_input');

		if (fileInput.files.length > 0) {
			const file = fileInput.files[0];
			uploadFile(file);
		}
	});
});

async function uploadFile(file) {
	if (!file) {
		return;
	}

	const fileExtension = file.name.split('.').pop();

	try {
		const response = await fetch('/upload', {
			method: 'POST',
			headers: {
				'File-Extension': fileExtension
			},
			body: file
		});

		if (!response.ok) {
			throw new Error(`Server responded with ${response.status}: ${response.statusText}`);
		}

		const result = await response.text();

		let url = `${window.location.protocol}//${window.location.hostname}${window.location.port ? ':' + window.location.port : ''}/${result}`;

		if (navigator.clipboard) {
			navigator.clipboard.writeText(url);
		}

		window.location.href = url;
	} catch (error) {
		console.error(error);
		alert(error.message);
	}
}
