class HtmlMain {

    public static layout(): string {
        return '<div id="main"></div>' +
            HtmlStatusBar.layout()
            + HtmlDialog.layout();
    }


    public static generateInit(model: INonInitializedModel): string {

        var result = '<h1>Remains tables to load: ' + model.tablesRemains + '</h1><h2>Total loading time is: ' + this.formatSeconds(model.initializingSeconds) + '</h2>' +
            '<table class="table table-striped table-bordered"><tr><th>TableName</th><th>Partitions loaded</th><th>Partitions total</th><th>Time gone</th><th>Time estimation</th></tr>'

        for (let itm of model.progress.sort((a, b) => a.tableName > b.tableName ? 1 : -1)) {
            result += '<tr><td style="width:50%">' + itm.tableName + '</td><td>' + itm.loaded + '</td><td>' + itm.toLoad + "</td><td>" + this.formatSeconds(itm.secondsGone) + "</td><td>" + this.getInitRemains(itm) + "</td></tr>"
        }

        return result + "</table>";

    }

    static getInitRemains(progress: ITableLoadProgress): String {

        if (progress.toLoad == 0 || progress.loaded == 0) {
            return "Unknown"
        }


        let pieceDuration = progress.secondsGone / progress.loaded;

        let remains = (progress.toLoad - progress.loaded) * pieceDuration;

        remains = this.trunc(remains);

        return this.formatSeconds(remains);

    }


    public static formatSecMin(value: number): String {
        if (value < 10) {
            return "0" + value.toFixed(0);
        }

        return value.toFixed(0);
    }

    public static trunc(value: number): number {

        let result = value.toFixed(2);

        let pos = result.indexOf('.');

        if (pos < 0) {
            pos = result.indexOf(',');
        }

        return parseInt(result.substring(0, pos))

    }


    public static formatSeconds(seconds: number): String {

        let hours = 0;
        if (seconds >= 3600) {
            hours = this.trunc(seconds / 3600);
            seconds -= hours * 3600;
        }

        let mins = 0;

        if (seconds >= 60) {
            mins = this.trunc(seconds / 60);
            seconds -= mins * 60;
        }

        return this.formatSecMin(hours) + ":" + this.formatSecMin(mins) + ":" + this.formatSecMin(seconds);
    }


}