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
        if (this.params.flagged_dkim === "true" || this.params.flagged_dkim === "false") {
            urlParams.push("flagged_dkim=" + this.params.flagged_dkim);
        }
        if (this.params.flagged_spf === "true" || this.params.flagged_spf === "false") {
            urlParams.push("flagged_spf=" + this.params.flagged_spf);
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
                    html`Filter active! <a class="ml button" href="#/reports">Show all Reports</a>` :
                    html`Filters:
                        <a class="ml button mr-5" href="#/reports?flagged=true">Reports with Problems</a>
                        <a class="button mr-5" href="#/reports?flagged_dkim=true">Reports with DKIM Problems</a>
                        <a class="button mr-5" href="#/reports?flagged_spf=true">Reports with SPF Problems</a>
                    `
                }
            </div>
            <dmarc-report-table .reports="${this.reports}"></dmarc-report-table>
        `;
    }
}

customElements.define("dmarc-reports", Reports);
