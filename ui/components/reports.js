import { LitElement, html, css } from "lit";

export class Reports extends LitElement {
    static styles = css`
        th {
            text-align: left;
        }

        td, th {
            padding-right: 10px;
        }
    `;

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
        return html`
            <table>
                <tr>
                    <th>ID</th>
                    <th>Org</th>
                    <th>Rows</th>
                    <th>Begin</th>
                    <th>End</th>
                </tr>
                ${this.reports.map((report) =>
                    html`<tr>
                        <td><a href="#/reports/${report.id}">${report.id}</a></td>
                        <td>${report.org}</td>
                        <td>${report.rows}</td>
                        <td>${new Date(report.date_begin * 1000).toLocaleString()}</td>
                        <td>${new Date(report.date_end * 1000).toLocaleString()}</td>
                    </tr>`
                )}
            </table>
        `;
    }
}

customElements.define("dmarc-reports", Reports);