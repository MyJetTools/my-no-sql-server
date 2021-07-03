var HtmlMain = /** @class */ (function () {
    function HtmlMain() {
    }
    HtmlMain.layout = function () {
        return '<div id="main"></div>' +
            HtmlStatusBar.layout()
            + HtmlDialog.layout();
    };
    return HtmlMain;
}());
//# sourceMappingURL=HtmlMain.js.map