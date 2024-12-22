import { LitElement, html } from "lit";
import { globalStyle } from "./style.js";

export class Reports extends LitElement {
    static styles = [globalStyle];

    static properties = {
        params: { type: Object },
        reports: { type: Array },
    };

    constructor() {
        super();
        this.params = {};
        this.reports = [];
        this.filtered = false;
    }

    updated(changedProperties) {
        if (changedProperties.has("params")) {
            this.updateReports();
        }
    }

    async updateReports() {
        const urlParams = [];
        if (this.params.flagged === "true" || this.params.flagged === "false") {
            urlParams.push("flagged=" + this.params.flagged);
        }
        if (this.params.domain) {
            urlParams.push("domain=" + encodeURIComponent(this.params.domain));
        }
        if (this.params.org) {
            urlParams.push("org=" + encodeURIComponent(this.params.org));
        }
        let url = "reports";
        if (urlParams.length > 0) {
            url += "?" + urlParams.join("&");
        }
        const response = await fetch(url);
        this.reports = await response.json();
        this.reports.sort((a, b) => b.date_begin - a.date_begin);
        this.filtered = this.filtered = urlParams.length > 0;
    }

    render() {
        return html`
            <h1>Reports</h1>
            <div>
                ${this.filtered ?
                    html`Filter active! Go back and <a href="#/reports">Show all Reports</a>` :
                    html`<a href="#/reports?flagged=true">Show only Reports with Problems</a>`
                }
            </div>
            <dmarc-report-table .reports="${this.reports}"></dmarc-report-table>
        `;
    }
}

customElements.define("dmarc-reports", Reports);
