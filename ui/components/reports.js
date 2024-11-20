import { LitElement, html, css } from "lit";

export class Reports extends LitElement {
    static properties = {
        reports: { type: Array },
        flaggedOnly: { type: Boolean },
    };

    constructor() {
        super();
        this.reports = [];
        this.flaggedOnly = false;
        this.updateReports();
    }

    toggleFlagged() {
        this.flaggedOnly = !this.flaggedOnly;
        this.updateReports();
    }

    async updateReports() {
        const url = "reports" + (this.flaggedOnly ? "?flagged=true" : "");
        const response = await fetch(url);
        this.reports = await response.json();
        this.reports.sort((a, b) => b.date_begin - a.date_begin);
    }

    render() {
        return html`
            <p>
                <button @click=${this.toggleFlagged}>
                    ${this.flaggedOnly ? "Show all Reports" : "Show only Reports with Problems"}
                </button>
            </p>
            <dmarc-report-table .reports="${this.reports}"></dmarc-report-table>
        `;
    }
}

customElements.define("dmarc-reports", Reports);
