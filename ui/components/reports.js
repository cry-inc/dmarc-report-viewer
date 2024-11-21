import { LitElement, html, css } from "lit";

export class Reports extends LitElement {
    static properties = {
        params: { type: Object },
        reports: { type: Array },
    };

    constructor() {
        super();
        this.params = {};
        this.reports = [];
    }

    updated(changedProperties) {
        if (changedProperties.has("params")) {
            this.updateReports();
        }
    }

    async updateReports() {
        const url = "reports" + (this.params.flagged ? "?flagged=true" : "");
        const response = await fetch(url);
        this.reports = await response.json();
        this.reports.sort((a, b) => b.date_begin - a.date_begin);
    }

    render() {
        return html`
            <p>
                ${this.params.flagged ?
                    html`<a href="#/reports">Show all Reports</a>` :
                    html`<a href="#/reports?flagged=true">Show only Reports with Problems</a>`
                }
            </p>
            <dmarc-report-table .reports="${this.reports}"></dmarc-report-table>
        `;
    }
}

customElements.define("dmarc-reports", Reports);
