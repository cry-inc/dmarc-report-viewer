import { LitElement, html } from "lit";
import { globalStyle } from "../style.js";

export class DmarcReportTable extends LitElement {
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

    renderProblemBadges(dkim, spf) {
        const badges = [];
        if (dkim) {
            badges.push(html`<span class="badge badge-negative mr-5">DKIM</span>`);
        }
        if (spf) {
            badges.push(html` <span class="badge badge-negative">SPF</span>`);
        }
        return badges;
    }

    render() {
        return html`
            <table>
                <tr>
                    <th class="help" title="Report ID, might be incomplete! Check details for full report ID.">ID</th>
                    <th class="xs-hidden">Organization</th>
                    <th class="sm-hidden">Domain</th>
                    <th class="help" title="Reports with SPF or DKIM problems are highlighted in red">Problems</th>
                    <th class="sm-hidden">Records</th>
                    <th class="md-hidden">Begin</th>
                    <th class="md-hidden">End</th>
                </tr>
                ${this.reports.length !== 0 ? this.reports.map((report) =>
                    html`<tr>
                            <td><a href="#/dmarc-reports/${report.hash}" title="${report.id}">${this.prepareId(report.id)}</a></td>
                            <td class="xs-hidden"><a href="#/dmarc-reports?org=${encodeURIComponent(report.org)}">${report.org}</a></td>
                            <td class="sm-hidden"><a href="#/dmarc-reports?domain=${encodeURIComponent(report.domain)}">${report.domain}</a></td>
                            <td>${this.renderProblemBadges(report.flagged_dkim, report.flagged_spf)}</td>
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

customElements.define("drv-dmarc-report-table", DmarcReportTable);
