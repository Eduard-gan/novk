function ajax_add_to_playlist(_instance) {
    var song_id = $('#add_into_playlist').data('invoked_song_id');
    var playlist_id = $(_instance).data('playlistId');

    $.ajax({
        url: '/music/ajax/playlist_operations',
        type: 'POST',
        data: {
            operation_type: 'add',
            song_id: song_id,
            playlist_id: playlist_id
        },
        success: function(response) {
            alert(response);
        }
    });
}
