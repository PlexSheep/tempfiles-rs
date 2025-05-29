// for settingsGenerateToken
document.addEventListener('DOMContentLoaded', function() {
	const form = document.getElementById('settingsGenerateToken');
	const tokenOutput = document.getElementById('tokenOutput');
	const tokenDuration = document.getElementById('tokenDuration');
	const tokenName = document.getElementById('tokenName');

	form.addEventListener('submit', async function(e) {
		e.preventDefault();

		if (tokenName.value.length < 5) {
			tokenOutput.value = 'Name is too short (at least 5 characters)';
			return;
		}
		if (tokenName.value.length > 40) {
			tokenOutput.value = 'Name is too long (at most 40 characters)';
			return;
		}

		// Show loading state
		tokenOutput.value = 'Generating token...';

		try {
			// Make API request
			const response = await fetch('/api/v1/auth/token', {
				method: 'POST',
				body: new URLSearchParams({
					'tokenDuration': tokenDuration.value,
					'tokenName': tokenName.value,
				})
			});

			if (!response.ok) {
				if (response.status == 409) {
					tokenOutput.value = 'Another Token already has this name. Use a different name.';
					return;
				}
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
			window.location.reload();
		}
	});
});

// for tokenListing
document.addEventListener('DOMContentLoaded', function() {
	const tokenDeleters = document.getElementsByClassName("tokenDeleter");

	// Convert HTMLCollection to Array and add event listeners
	Array.from(tokenDeleters).forEach(deleteButton => {
		deleteButton.addEventListener('click', async function(e) {
			e.preventDefault();

			const tokenName = this.getAttribute('for');

			// Show confirmation dialog
			if (!confirm(`Are you sure you want to delete token "${tokenName}"? This action cannot be undone.`)) {
				return;
			}

			// Show loading state
			const originalText = this.textContent;
			this.textContent = 'Deleting...';
			this.disabled = true;

			try {
				// Make DELETE request
				const response = await fetch(`/api/v1/auth/token/${encodeURIComponent(tokenName)}`, {
					method: 'DELETE',
				});

				if (!response.ok) {
					throw new Error(`HTTP error! status: ${response.status}`);
				}

				// Remove the token listing from the DOM
				const tokenListing = this.closest('.tokenListing');
				if (tokenListing) {
					tokenListing.style.opacity = '0.5';
					setTimeout(() => {
						tokenListing.remove();
					}, 300);
				}

				console.log(`Token "${tokenName}" deleted successfully`);

			} catch (error) {
				console.error('Error deleting token:', error);
				alert(`Failed to delete token "${tokenName}". Please try again.`);

				// Restore button state
				this.textContent = originalText;
				this.disabled = false;
			}
		});
	});
});
