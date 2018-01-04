Mode = 'normal';


function ccolorizeSong(ColorizingTarget) {
	var PreviousSongList = document.getElementsByClassName("warning");
    if (PreviousSongList.length > 0) {
      	PreviousSongList[0].className = "success";
       	ColorizingTarget.parentNode.className = "warning";
	}
	else{
       	ColorizingTarget.parentNode.className = "warning";
	}
}


function clickWrapper(ChosenSong) {
    var Path = getSongPath(ChosenSong.id);
    var Player = document.getElementById("player");
	Player.src = Path;
	Player.play();
	ccolorizeSong(ChosenSong);

	if (Mode === 'repeat')
	{
		alert('Mode is "repeat');
	}
	else
    {
		Player.onended = function() { changeSong( Player , ChosenSong.id ) };
	}
}


function getSongPath( SongID ) {
	var Song = document.getElementById(SongID);
	var ReturningPath = Song.getAttribute("data-path");
	ReturningPath = ReturningPath.replace( /^/ , "/static/uploaded/" );
    return( ReturningPath );
}


function getNextSongID( SongID ) {
	var Playlist = document.getElementsByClassName("Song");
	for( var i = 0; i < Playlist.length; i++ ) {
		if( Playlist[i].classList.contains("Song") && Playlist[i].id === SongID ) {
			if( i+1 === Playlist.length ){
				return( Playlist[0].id )
			}
			else{
				return( Playlist[i+1].id );
			}
		}
	}
	return( "getNextSongID ended with no result" );
}


function changeSong( Player , EndedSong ) {
	var NewSongID = getNextSongID(EndedSong);
	Player.src = getSongPath(NewSongID);
	Player.load();
	Player.onended = function() { changeSong(Player, NewSongID) };
	Player.play();
	ccolorizeSong(document.getElementById(NewSongID));
}


(function ($, window) {

    $.fn.contextMenu = function (settings) {

        return this.each(function () {

            // Open context menu
            $(this).on("contextmenu", function (e) {
                // return native menu if pressing control
                if (e.ctrlKey) return;

                //open menu
                var $menu = $(settings.menuSelector)
                    .data("invokedOn", $(e.target))
                    .show()
                    .css({
                        position: "absolute",
                        left: getMenuPosition(e.clientX, 'width', 'scrollLeft'),
                        top: getMenuPosition(e.clientY, 'height', 'scrollTop')
                    })
                    .off('click')
                    .on('click', 'a', function (e) {
                        $menu.hide();

                        var $invokedOn = $menu.data("invokedOn");
                        var $selectedMenu = $(e.target);

                        settings.menuSelected.call(this, $invokedOn, $selectedMenu);
                    });

                return false;
            });

            //make sure menu closes on any click
            $('body').click(function () {
                $(settings.menuSelector).hide();
            });
        });

        function getMenuPosition(mouse, direction, scrollDir) {
            var win = $(window)[direction](),
                scroll = $(window)[scrollDir](),
                menu = $(settings.menuSelector)[direction](),
                position = mouse + scroll;

            // opening menu would pass the side of the page
            if (mouse + menu > win && menu < mouse)
                position -= menu;

            return position;
        }

    };
})(jQuery, window);

$("#SongTable td").contextMenu({
    menuSelector: "#contextMenu",
    menuSelected: function (invokedOn, selectedMenu) {
        var msg = "You selected the menu item '" + selectedMenu.text() +
            "' on the value '" + invokedOn.text() + "'";
        alert(msg);
    }
});