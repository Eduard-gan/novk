async function ajax_add_to_playlist(instance) {
    const song_id = $('#add_into_playlist').data('invoked_song_id');
    const playlist_id = instance.dataset.playlistId;
    try {
        const response = await fetch('/music/ajax/playlist-operations', {
            method: 'POST', credentials: 'same-origin',
            body: new URLSearchParams({operation_type: 'add', song_id, playlist_id})
        });
        if (response.redirected) {
            window.location.assign(response.url);
        } else if (response.status === 409) {
            alert((await response.json()).detail);
        } else if (response.ok) {
            alert(await response.text());
        } else {
            alert('Не удалось добавить песню.');
        }
    } catch (e) {
        alert('Не удалось отправить запрос, проверьте соединение.');
    }
}
