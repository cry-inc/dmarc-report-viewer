import { LitElement, html } from "lit";
import { globalStyle } from "../style.js";
import { join } from "../utils.js";

export class TlsReportTable extends LitElement {
    static styles = [globalStyle];

    static properties = {
        reports: { type: Array },
    };

    constructor() {
        super();
        this.reports = [];
    }

    prepareId(id) {
        // Many reports contain a piece of the timestamp that is always the same,
        // lets remove it to save some screen space!
        const shortened = id.replace("T00:00:00Z", "");

        const limit = 25;
        if (shortened.length <= limit) {
            return shortened;
        } else {
            return shortened.substring(0, limit) + "...";
        }
    }

    renderProblemBadges(sts, tlsa) {
        const badges = [];
        if (sts) {
            badges.push(html`<span class="badge badge-negative">MTA-STS</span>`);
        }
        if (tlsa) {
            badges.push(html`<span class="badge badge-negative">TLSA</span>`);
        }
        return join(badges, html` `);
    }

    renderDomains(domains) {
        if (domains.length === 0) {
            return html`<span class="faded">No domains</span>`;
        }
        const links = domains.map(
            d => html`<a href="#/tls-reports?domain=${encodeURIComponent(d)}">${d}</a>`
        );
        return join(links, html`, `);
    }

    render() {
        return html`
            <table>
                <tr>
                    <th class="help" title="Report ID, might be incomplete! Check details for full report ID.">ID</th>
                    <th class="xs-hidden">Organization</th>
                    <th class="sm-hidden">Domains</th>
                    <th class="help" title="Reports with MTA-STS or TLSA problems are highlighted in red">Problems</th>
                    <th class="sm-hidden">Policies</th>
                    <th class="md-hidden">Begin</th>
                    <th class="md-hidden">End</th>
                </tr>
                ${this.reports.length !== 0 ? this.reports.map((report) =>
                    html`<tr>
                            <td><a href="#/tls-reports/${report.hash}" title="${report.id}">${this.prepareId(report.id)}</a></td>
                            <td class="xs-hidden"><a href="#/tls-reports?org=${encodeURIComponent(report.org)}">${report.org}</a></td>
                            <td class="sm-hidden">${this.renderDomains(report.domains)}</td>
                            <td>${this.renderProblemBadges(report.flagged_sts, report.flagged_tlsa)}</td>
                            <td class="sm-hidden">${report.records}</td>
                            <td class="md-hidden">${new Date(report.date_begin).toLocaleString()}</td>
                            <td class="md-hidden">${new Date(report.date_end).toLocaleString()}</td>
                        </tr>`

                ) : html`<tr>
                            <td colspan="7">No reports found.</td>
                        </tr>`
                }
            </table>
        `;
    }
}

customElements.define("drv-tls-report-table", TlsReportTable);
