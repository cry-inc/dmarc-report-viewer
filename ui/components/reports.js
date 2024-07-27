import { LitElement, html, css } from "lit";

export class Reports extends LitElement {
    static styles = css`
        table {
            width: 100%;
        }

        th {
            text-align: left;
            background-color: #efefef;
        }

        td, th {
            padding-left: 5px;
            padding-right: 10px;
            padding-top: 3px;
            padding-bottom: 3px;
        }

        tr:hover {
            background-color: #f4f4f4;
        }

        a {
            color: rgb(14, 117, 212);
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
                    <th>Organization</th>
                    <th>Domain</th>
                    <th>Records</th>
                    <th>Begin</th>
                    <th>End</th>
                </tr>
                ${this.reports.map((report) =>
            html`<tr>
                        <td><a href="#/reports/${report.id}">${report.id}</a></td>
                        <td>${report.org}</td>
                        <td>${report.domain}</td>
                        <td>${report.records}</td>
                        <td>${new Date(report.date_begin * 1000).toLocaleString()}</td>
                        <td>${new Date(report.date_end * 1000).toLocaleString()}</td>
                    </tr>`
        )}
            </table>
        `;
    }
}

customElements.define("dmarc-reports", Reports);
