// for settingsGenerateToken
document.addEventListener('DOMContentLoaded', function() {
	const form = document.getElementById('settingsGenerateToken');
	const tokenOutput = document.getElementById('tokenOutput');
	const tokenDuration = document.getElementById('tokenDuration');

	form.addEventListener('submit', async function(e) {
		e.preventDefault();

		// Show loading state
		tokenOutput.value = 'Generating token...';
		tokenOutput.disabled = true;

		try {
			// Make API request
			const response = await fetch('/api/v1/auth/token', {
				method: 'POST',
				body: new URLSearchParams({
					'tokenDuration': tokenDuration.value,
				})
			});

			if (!response.ok) {
				throw new Error(`HTTP error! status: ${response.status}`);
			}

			const data = await response.json();

			// Display the generated token
			if (data.token) {
				tokenOutput.value = data.token;
				tokenOutput.select(); // Select the token for easy copying
			} else {
				throw new Error('No token received from server');
			}

		} catch (error) {
			console.error('Error generating token:', error);
			tokenOutput.value = 'Error generating token. Please try again.';
		} finally {
			tokenOutput.disabled = false;
		}
	});
});
