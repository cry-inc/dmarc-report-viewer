import { LitElement, html, css } from "lit";
import { globalStyle } from "./style.js";

export class ReportTable extends LitElement {
    static styles = [globalStyle];

    static properties = {
        reports: { type: Array },
    };

    constructor() {
        super();
        this.reports = [];
    }

    prepareId(id) {
        const limit = 25;
        if (id.length <= limit) {
            return id;
        } else {
            return id.substring(0, limit) + "...";
        }
    }

    renderProblemsBadge(problems) {
        if (problems) {
            return html`<span class="badge badge-negative">Yes</span>`;
        } else {
            return html`<span class="faded">No</span>`;
        }
    }

    render() {
        return html`
            <table>
                <tr>
                    <th>ID</th>
                    <th class="xs-hidden">Organization</th>
                    <th class="sm-hidden">Domain</th>
                    <th class="help" title="Reports that indicate possible problems">Problems</th>
                    <th class="sm-hidden">Records</th>
                    <th class="md-hidden">Begin</th>
                    <th class="md-hidden">End</th>
                </tr>
                ${this.reports.length !== 0 ? this.reports.map((report) =>
                    html`<tr>
                            <td><a href="#/reports/${report.hash}" title="${report.id}">${this.prepareId(report.id)}</a></td>
                            <td class="xs-hidden"><a href="#/reports?org=${encodeURIComponent(report.org)}">${report.org}</a></td>
                            <td class="sm-hidden"><a href="#/reports?domain=${encodeURIComponent(report.domain)}">${report.domain}</a></td>
                            <td>${this.renderProblemsBadge(report.flagged)}</td>
                            <td class="sm-hidden">${report.records}</td>
                            <td class="md-hidden">${new Date(report.date_begin * 1000).toLocaleString()}</td>
                            <td class="md-hidden">${new Date(report.date_end * 1000).toLocaleString()}</td>
                        </tr>`

                ) : html`<tr>
                            <td colspan="7">No reports found.</td>
                        </tr>`
                }
            </table>
        `;
    }
}

customElements.define("dmarc-report-table", ReportTable);
