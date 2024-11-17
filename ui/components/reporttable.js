import { LitElement, html, css } from "lit";

export class ReportTable extends LitElement {
    static styles = css`
        table {
            width: 100%;
        }

        th {
            text-align: left;
            background-color: #efefef;
        }

        td, th {
            padding-left: 10px;
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

        .noproblem {
            color: #ccc;
        }

        .problem {
            border-radius: 3px;
            padding-left: 4px;
            padding-right: 4px;
            color: white;
            background-color: #f00;
        }
    `;

    static properties = {
        reports: { type: Array },
    };

    constructor() {
        super();
        this.reports = [];
    }

    renderProblemsBadge(problems) {
        if (problems) {
            return html`<span class="problem">Yes</span>`;
        } else {
            return html`<span class="noproblem">No</span>`;
        }
    }

    render() {
        return html`
            <table>
                <tr>
                    <th>ID</th>
                    <th>Organization</th>
                    <th>Domain</th>
                    <th>Problems</th>
                    <th>Records</th>
                    <th>Begin</th>
                    <th>End</th>
                </tr>
                ${this.reports.map((report) =>
                    html`<tr>
                        <td><a href="#/reports/${report.id}">${report.id}</a></td>
                        <td>${report.org}</td>
                        <td>${report.domain}</td>
                        <td>${this.renderProblemsBadge(report.flagged)}</td>
                        <td>${report.records}</td>
                        <td>${new Date(report.date_begin * 1000).toLocaleString()}</td>
                        <td>${new Date(report.date_end * 1000).toLocaleString()}</td>
                    </tr>`
                )}
            </table>
        `;
    }
}

customElements.define("dmarc-report-table", ReportTable);
