import { LitElement, html, css } from "lit";

export class Reports extends LitElement {
    static properties = {
        reports: { type: Array },
    };

    constructor() {
        super();
        this.reports = [];
        this.updateReports();
    }

    async updateReports() {
        const response = await fetch("reports");
        this.reports = await response.json();
        this.reports.sort((a, b) => b.date_begin - a.date_begin);
    }

    render() {
        return html`<dmarc-report-table .reports="${this.reports}"></dmarc-report-table>`;
    }
}

customElements.define("dmarc-reports", Reports);
