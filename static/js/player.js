Mode = 'normal';

function clickWrapper(ChosenSong) {
    var Path = getSongPath(ChosenSong.id);
    var Player = document.getElementById("player");
	Player.src = Path;
	Player.play();

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

function appendTextToMusic(Text) {
	var Log = document.getElementsByTagName("h6");
	var node = document.createElement("LI");
    var textnode = document.createTextNode(Text);
    node.appendChild(textnode);
    Log.item(0).appendChild(node);
}

function changeSong( Player , EndedSong ) {
	appendTextToMusic("changeSong: Entered, getting  NewSongID...");
	var NewSongID = getNextSongID(EndedSong);
	appendTextToMusic("changeSong: Got NewSongID("+NewSongID+") Setting new Player.src path...");
	Player.src = getSongPath(NewSongID);
	appendTextToMusic("changeSong: New Player.src path set = "+Player.src+". Trying to call load()...");
	Player.load();
	appendTextToMusic("changeSong: load() called. Setting updated onended event function...");
	Player.onended = function() { changeSong(Player, NewSongID) };
	appendTextToMusic("changeSong: onended set. Trying to call play()...");
	Player.play();
	appendTextToMusic("changeSong: play() called. Function Ended. WTF: Next will be call 'onended' on 59Line");
}
