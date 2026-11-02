for (const form of document.querySelectorAll('[data-fetch-form]')) {
    form.addEventListener('submit', async (event) => {
        event.preventDefault();
        const error = form.querySelector('[data-form-error]');
        error.textContent = '';
        for (const field of form.querySelectorAll('[data-field-error]')) field.textContent = '';
        const file = form.querySelector('input[type=file]');
        if (file && file.files[0] && file.files[0].size > Number(form.dataset.fileLimit)) {
            error.textContent = 'Файл больше 20 МБ.';
            return;
        }
        const button = form.querySelector('button[type=submit]');
        button.disabled = true;
        try {
            const data = new FormData(form);
            const response = await fetch(form.action, {
                method: 'POST', credentials: 'same-origin',
                body: file ? data : new URLSearchParams(data)
            });
            if (response.status === 409) {
                const problem = await response.json();
                const field = problem.field && form.querySelector(`[data-field-error="${problem.field}"]`);
                (field || error).textContent = problem.detail;
            } else if (response.redirected) {
                window.location.assign(response.url);
            } else if (!response.ok) {
                error.textContent = 'Не удалось выполнить операцию.';
            }
        } catch (e) {
            error.textContent = file ? 'Не удалось отправить файл, проверьте размер и соединение.' : 'Не удалось отправить форму, проверьте соединение.';
        } finally {
            button.disabled = false;
        }
    });
}
