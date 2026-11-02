// Menu functions
(function ($, window) {
    $.fn.contextMenu = function (settings) {
        return this.each(function () {
            $(this).on("contextmenu", function (e) {
                if (e.ctrlKey) return;

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

            $('body').click(function () {
                $(settings.menuSelector).hide();
                return true;
            });
        });

        function getMenuPosition(mouse, direction, scrollDir) {
            var win = $(window)[direction](),
                scroll = $(window)[scrollDir](),
                menu = $(settings.menuSelector)[direction](),
                position = mouse + scroll;

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

        if (selectedMenu[0].text == 'Добавить в плэйлист') {
            $('#add_into_playlist').data({'invoked_song_id': invokedOn.attr('id')});
            bootstrap.Modal.getOrCreateInstance(document.getElementById('add_into_playlist')).show();
        }
        else {
            alert(msg);
        }
    }
});
