import { LitElement, html } from "lit";
import { globalStyle } from "../style.js";

export class TlsRptReports extends LitElement {
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
        if (this.params.flagged_sts === "true" || this.params.flagged_sts === "false") {
            urlParams.push("flagged_sts=" + this.params.flagged_sts);
        }
        if (this.params.flagged_tlsa === "true" || this.params.flagged_tlsa === "false") {
            urlParams.push("flagged_tlsa=" + this.params.flagged_tlsa);
        }
        if (this.params.domain) {
            urlParams.push("domain=" + encodeURIComponent(this.params.domain));
        }
        if (this.params.org) {
            urlParams.push("org=" + encodeURIComponent(this.params.org));
        }
        let url = "tlsrpt-reports";
        if (urlParams.length > 0) {
            url += "?" + urlParams.join("&");
        }
        const response = await fetch(url);
        this.reports = await response.json();
        this.reports.sort((a, b) => new Date(b.date_begin) - new Date(a.date_begin));
        this.filtered = this.filtered = urlParams.length > 0;
    }

    render() {
        return html`
            <h1>SMTP TLS Reports</h1>
            <div>
                ${this.filtered ?
                    html`Filter active! <a class="ml button" href="#/tlsrpt-reports">Show all Reports</a>` :
                    html`Filters:
                        <a class="ml button mr-5" href="#/tlsrpt-reports?flagged=true">Reports with Problems</a>
                        <a class="button mr-5" href="#/tlsrpt-reports?flagged_sts=true">Reports with MTA-STS Problems</a>
                        <a class="button mr-5" href="#/tlsrpt-reports?flagged_tlsa=true">Reports with TLSA Problems</a>
                    `
                }
            </div>
            <drv-tlsrpt-report-table .reports="${this.reports}"></drv-tlsrpt-report-table>
        `;
    }
}

customElements.define("drv-tlsrpt-reports", TlsRptReports);
