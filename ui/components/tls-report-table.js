import { LitElement, html } from "lit";
import { globalStyle } from "../style.js";
import { join } from "../utils.js";

export class TlsReportTable extends LitElement {
    static styles = [globalStyle];

    static properties = {
        reports: { type: Array },
        filterCallback: { attribute: false },
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

    renderFilterLink(filter, value) {
        const href = `#/tls-reports?${filter}=${encodeURIComponent(value)}`;
        return html`<a href="${href}" @click="${this.filterCallback ? (e) => {
            e.preventDefault();
            this.filterCallback(filter, value);
        } : undefined}">${value}</a>`;
    }

    renderDomains(domains) {
        if (domains.length === 0) {
            return html`<span class="faded">No domains</span>`;
        }
        const links = domains.map(domain => this.renderFilterLink("domain", domain));
        return join(links, html`, `);
    }

    render() {
        return html`
            <table>
                <tr>
                    <th class="help" title="Report ID, might be incomplete! Check details for full report ID.">ID</th>
                    <th class="xs-hidden">Organization</th>
                    <th class="sm-hidden">Domains</th>
                    <th>Problems</th>
                    <th class="lg-hidden help" title="Number of policies in the report">Policies</th>
                    <th class="lg-hidden help" title="Sum of all failure counts for all policies in the report">Failures</th>
                    <th class="md-hidden">Begin</th>
                    <th class="md-hidden">End</th>
                </tr>
                ${this.reports.length !== 0 ? this.reports.map((report) =>
                    html`<tr>
                            <td><a href="#/tls-reports/${report.hash}" title="${report.id}">${this.prepareId(report.id)}</a></td>
                            <td class="xs-hidden">${this.renderFilterLink("org", report.org)}</td>
                            <td class="sm-hidden">${this.renderDomains(report.domains)}</td>
                            <td>${this.renderProblemBadges(report.flagged_sts, report.flagged_tlsa)}</td>
                            <td class="lg-hidden">${report.records}</td>
                            <td class="lg-hidden">${report.failures === 0 ? html`<span class="faded">${report.failures}</span>` : report.failures}</td>
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
