import { LitElement, html, css } from "lit";
import { globalStyle } from "../style.js";

export class DmarcReport extends LitElement {
    static styles = [globalStyle];

    static get properties() {
        return {
            id: { type: String },
            mailId: { type: String, attribute: false },
        };
    }

    constructor() {
        super();
        this.id = null;
        this.mailId = null;
        this.report = null;
        this.ip2dns = {};
        this.ip2location = {};
        this.ipDetails = {};
    }

    async updated(changedProperties) {
        if (changedProperties.has("id") && changedProperties.id !== this.id && this.id) {
            const response = await fetch("dmarc-reports/" + this.id);
            const rwi = await response.json();
            this.report = rwi.report;
            this.mailId = rwi.mail_id;
        }
    }

    async lookupIp(ip) {
        if (this.ipDetails[ip]) {
            this.ipDetails[ip] = false;
        } else {
            this.ipDetails[ip] = true;
            this.getDnsForIp(ip);
            this.getLocationForIp(ip);
        }
        this.requestUpdate();
    }

    async getDnsForIp(ip) {
        const response = await fetch("ips/" + ip + "/dns");
        if (response.status === 200) {
            const result = await response.text();
            this.ip2dns[ip] = result;
        } else {
            this.ip2dns[ip] = null;
        }
        this.requestUpdate();
    }

    async getLocationForIp(ip) {
        const response = await fetch("ips/" + ip + "/location");
        if (response.status === 200) {
            const result = await response.json();
            this.ip2location[ip] = result;
        } else {
            this.ip2location[ip] = null;
        }
        this.requestUpdate();
    }

    renderOptional(value) {
        if (value !== null && value !== undefined) {
            return html`${value}`;
        } else {
            return html`<span class="faded">n/a</span>`;
        }
    }

    renderResultBadge(result) {
        if (result === "fail" || result === "temperror" ||
            result === "permerror" || result === "softfail" ||
            result === "quarantine" || result === "reject"
        ) {
            return html`<span class="badge badge-negative">${result}</span>`;
        } else if (result === "pass") {
            return html`<span class="badge badge-positive">${result}</span>`;
        } else if (result !== null || result !== undefined) {
            return html`<span class="faded">n/a</span>`;
        } else {
            return html`<span class="badge">${result}</span>`;
        }
    }

    renderLocation(lat, lon) {
        if (lat === undefined || lon === undefined) {
            return html`<span class="faded">n/a</span>`;
        }
        return html`<a target="_blank" title="Show on OpenStreeMap" href="https://www.openstreetmap.org/#map=8/${lat}/${lon}">${lat}, ${lon}</a>`;
    }

    renderPropIfObjDefined(obj, prop) {
        if (obj === undefined) {
            return html`<span class="faded">loading...</span>`;
        } else if (obj) {
            return obj[prop]
        } else {
            return html`<span class="faded">n/a</span>`;
        }
    }

    renderIfDefined(obj) {
        if (obj === undefined) {
            return html`<span class="faded">loading...</span>`;
        } else if (obj) {
            return obj;
        } else {
            return html`<span class="faded">n/a</span>`;
        }
    }

    render() {
        if (!this.report) {
            return html`No report loaded`;
        }

        let errors = null;
        if (this.report.report_metadata.error) {
            errors = this.report.report_metadata.error.join(", ");
        }

        return html`
            <h1>Report Details</h1>
            <p>
                <a class="button" href="#/mails/${this.mailId}">Show Mail</a>
                <a class="button" href="/dmarc-reports/${this.id}/xml" target="_blank">Open XML</a>
                <a class="button" href="/dmarc-reports/${this.id}/json" target="_blank">Open JSON</a>
            </p>
            <table>
                <tr>
                    <th colspan="2">Report Header</td>
                </tr>
                <tr>
                    <td class="name">ID</td>
                    <td>${this.report.report_metadata.report_id}</td>
                </tr>
                <tr>
                    <td class="name">Organization</td>
                    <td>${this.report.report_metadata.org_name}</td>
                </tr>
                <tr>
                    <td class="name">Records</td>
                    <td>${this.report.record.length}</td>
                </tr>
                <tr>
                    <td class="name">Date Range Begin</td>
                    <td>${new Date(this.report.report_metadata.date_range.begin * 1000).toLocaleString()}</td>
                </tr>
                <tr>
                    <td class="name">Date Range End</td>
                    <td>${new Date(this.report.report_metadata.date_range.end * 1000).toLocaleString()}</td>
                </tr>
                <tr>
                    <td class="name">E-Mail</td>
                    <td>${this.report.report_metadata.email}</td>
                </tr>
                <tr>
                    <td class="name">Extra Contact Info</td>
                    <td>${this.renderOptional(this.report.report_metadata.extra_contact_info)}</td>
                </tr>
                <tr>
                    <td class="name">Errors</td>
                    <td>${this.renderOptional(errors)}</td>
                </tr>
                <tr>
                    <td class="name">Version</td>
                    <td>${this.renderOptional(this.report.version)}</td>
                </tr>
                <tr>
                    <th colspan="2">Published Policy</th>
                </tr>
                <tr>
                    <td class="name">Domain</td>
                    <td>${this.report.policy_published.domain}</td>
                </tr>
                <tr>
                    <td class="name help" title="DKIM alignment mode">adkim</td>
                    <td>${this.renderOptional(this.report.policy_published.adkim)}</td>
                </tr>
                <tr>
                    <td class="name help" title="SPF alignment mode">aspf</td>
                    <td>${this.renderOptional(this.report.policy_published.aspf)}</td>
                </tr>
                <tr>
                    <td class="name help" title="Policy to apply to messages from the domain">p</td>
                    <td>${this.report.policy_published.p}</td>
                </tr>
                <tr>
                    <td class="name help" title="Policy to apply to messages from subdomains">sp</td>
                    <td>${this.renderOptional(this.report.policy_published.sp)}</td>
                </tr>
                <tr>
                    <td class="name help" title="Percent of messages to which policy applies">pct</td>
                    <td>${this.renderOptional(this.report.policy_published.pct)}</td>
                </tr>
                <tr>
                    <td class="name help" title="Failure reporting options in effect">fo</td>
                    <td>${this.renderOptional(this.report.policy_published.fo)}</td>
                </tr>
            </table>
            ${this.report.record.map((record) => html`
                <h2>Record</h2>
                <table>
                    <tr>
                        <th colspan="2">Record Header</td>
                    </tr>
                    <tr>
                        <td class="name">Source IP</td>
                        <td>
                            ${record.row.source_ip}
                            <button @click="${() => this.lookupIp(record.row.source_ip)}" class="button sm help" title="Search DNS hostname for IP and geolocate it">DNS and Location</button>
                            <a class="button sm help" title="Look up WHOIS record for IP and show in new tab" target="blank" href="/ips/${record.row.source_ip}/whois">WHOIS</a>
                        </td>
                    </tr>
                    <tbody class="sourceip" style="${this.ipDetails[record.row.source_ip] ? "": "display:none"}">
                        <tr>
                            <td class="name">Source IP DNS</td>
                            <td>${this.renderIfDefined(this.ip2dns[record.row.source_ip])}
                            </td>
                        </tr>
                        <tr>
                            <td class="name">Source IP Country</td>
                            <td>${this.renderPropIfObjDefined(this.ip2location[record.row.source_ip], "country")}</td>
                        </tr>
                        <tr>
                            <td class="name">Source IP City</td>
                            <td>${this.renderPropIfObjDefined(this.ip2location[record.row.source_ip], "city")}</td>
                        </tr>
                        <tr>
                            <td class="name">Source IP ISP</td>
                            <td>${this.renderPropIfObjDefined(this.ip2location[record.row.source_ip], "isp")}</td>
                        </tr>
                        <tr>
                            <td class="name">Source IP AS</td>
                            <td>${this.renderPropIfObjDefined(this.ip2location[record.row.source_ip], "as")}</td>
                        </tr>
                        <tr>
                            <td class="name help" title="Known Proxy, VPN or Tor exit address?">Source IP Proxy</td>
                            <td>${this.renderPropIfObjDefined(this.ip2location[record.row.source_ip], "proxy")}</td>
                        </tr>
                        <tr>
                            <td class="name help" title="Known data center, hosting or colocated">Source IP Data Center</td>
                            <td>${this.renderPropIfObjDefined(this.ip2location[record.row.source_ip], "hosting")}</td>
                        </tr>
                        <tr>
                            <td class="name">Source IP Location</td>
                            <td>${this.ip2location[record.row.source_ip] === undefined ?
                                    html`<span class="faded">loading</span>` :
                                    this.renderLocation(this.ip2location[record.row.source_ip]?.lat, this.ip2location[record.row.source_ip]?.lon)
                                }
                            </td>
                        </tr>
                    </tbody>
                    <tr>
                        <td class="name">Count</td>
                        <td>${record.row.count}</td>
                    </tr>
                    <tr>
                        <td class="name">Policy Disposition</td>
                        <td>${this.renderResultBadge(record.row.policy_evaluated.disposition)}</td>
                    </tr>
                    <tr>
                        <td class="name">Policy DKIM</td>
                        <td>${this.renderResultBadge(record.row.policy_evaluated.dkim)}</td>
                    </tr>
                    <tr>
                        <td class="name">Policy SPF</td>
                        <td>${this.renderResultBadge(record.row.policy_evaluated.spf)}</td>
                    </tr>
                    <tr>
                        <td class="name">Policy Reason</td>
                        <td>
                            ${record.row.policy_evaluated.reason ?
                                record.row.policy_evaluated.reason.map(
                                    (reason) => html`${reason.kind} ${reason.comment}`
                                ) : html`<span class="na">n/a</span>`
                            }
                        </td>
                    </tr>
                    <tr>
                        <td class="name">Header From</td>
                        <td>${record.identifiers.header_from}</td>
                    </tr>
                    <tr>
                        <td class="name">Envelope From</td>
                        <td>${this.renderOptional(record.identifiers.envelope_from)}</td>
                    </tr>
                    <tr>
                        <td class="name">Envelope To</td>
                        <td>${this.renderOptional(record.identifiers.envelope_to)}</td>
                    </tr>
                    ${record.auth_results.spf.map((result) => html`
                        <tr>
                            <th colspan="2">SPF Auth Result</td>
                        </tr>
                        <tr>
                            <td class="name">Domain</td>
                            <td>${result.domain}</td>
                        </tr>
                        <tr>
                            <td class="name">Scope</td>
                            <td>${this.renderOptional(result.scope)}</td>
                        </tr>
                        <tr>
                            <td class="name">Result</td>
                            <td>${this.renderResultBadge(result.result)}</td>
                        </tr>
                    `)}
                    ${(record.auth_results.dkim ?
                        record.auth_results.dkim : []).map((result) => html`
                        <tr>
                            <th colspan="2">DKIM Auth Result</td>
                        </tr>
                        <tr>
                            <td class="name">Domain</td>
                            <td>${result.domain}</td>
                        </tr>
                        <tr>
                            <td class="name">Scope</td>
                            <td>${this.renderOptional(result.selector)}</td>
                        </tr>
                        <tr>
                            <td class="name">Result</td>
                            <td>${this.renderResultBadge(result.result)}</td>
                        </tr>
                        <tr>
                            <td class="name">Human Result</td>
                            <td>${this.renderOptional(result.human_result)}</td>
                        </tr>
                    `)}
                </table>
            `)}
        `;
    }
}

customElements.define("drv-dmarc-report", DmarcReport);
