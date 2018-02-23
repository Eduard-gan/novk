function ajax_add_to_playlist(_instance) {
    var song_id = $('#add_into_playlist').data('invoked_song_id');
    var playlist_id = $(_instance).data('playlistNumber');
    var csrf_token = $('[name=csrfmiddlewaretoken]').val();

    $.ajax({
        url:'http://127.0.0.1:8000/music/ajax/playlist_operations/',
        type:'POST',
        headers: {"X-CSRFToken": csrf_token},
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