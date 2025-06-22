import { LitElement, html, nothing } from "lit";
import { globalStyle } from "../style.js";
import { join } from "../utils.js";

export class TlsRptReport extends LitElement {
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
            const response = await fetch("tlsrpt-reports/" + this.id);
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

    renderPolicyTypeBadge(result) {
        switch (result) {
            case "no-policy-found":
                return html`<span class="faded">No Policy Found</span>`;
            case "sts":
                return html`<span class="badge">MTA-STS</span>`;
            case "tlsa":
                return html`<span class="badge">TLSA</span>`;
        }
        return html`<span class="badge">${result}</span>`;
    }

    renderFailureCountBadge(count) {
        if (count === 0) {
            return html`<span class="badge badge-positive">0</span>`;
        } else {
            return html`<span class="badge badge-negative">${count}</span>`;
        }
    }

    renderFailureResultType(type) {
        switch (type) {
            case "starttls-not-supported":
                return html`STARTTLS not supported`;
            case "certificate-host-mismatch":
                return html`Certificate host mismatch`;
            case "certificate-expired":
                return html`Certificate expired`;
            case "certificate-not-trusted":
                return html`Certificate not trusted`;
            case "validation-failure":
                return html`Validation failure`;
            case "tlsa-invalid":
                return html`TLSA invalid`;
            case "dnssec-invalid":
                return html`DNSSEC invalid`;
            case "dane-required":
                return html`DANE required`;
            case "sts-policy-fetch-error":
                return html`MTA-STS policy fetch error`;
            case "sts-policy-invalid":
                return html`MTA-STS policy invalid`;
            case "sts-webpki-invalid":
                return html`MTA-STS WebPKI invalid`;
        }
        return html`${type}`;
    }

    renderMultilineCell(array) {
        const lines = array.map(l => html`${l}`);
        return join(lines, html`<br>`);
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

        return html`
            <h1>Report Details</h1>
            <p>
                <a class="button" href="#/mails/${this.mailId}">Show Mail</a>
                <a class="button" href="/tlsrpt-reports/${this.id}/json" target="_blank">Open JSON</a>
            </p>
            <table>
                <tr>
                    <th colspan="2">Report Header</td>
                </tr>
                <tr>
                    <td class="name">ID</td>
                    <td>${this.report["report-id"]}</td>
                </tr>
                <tr>
                    <td class="name">Organization</td>
                    <td>${this.report["organization-name"]}</td>
                </tr>
                <tr>
                    <td class="name">Evaluated Policies</td>
                    <td>${this.report.policies.length}</td>
                </tr>
                <tr>
                    <td class="name">Date Range Begin</td>
                    <td>${new Date(this.report["date-range"]["start-datetime"]).toLocaleString()}</td>
                </tr>
                <tr>
                    <td class="name">Date Range End</td>
                    <td>${new Date(this.report["date-range"]["end-datetime"]).toLocaleString()}</td>
                </tr>
                <tr>
                    <td class="name">Contact Info</td>
                    <td>${this.report["contact-info"]}</td>
                </tr>
            </table>

            ${this.report.policies.sort((a, b) => a.policy["policy-type"].localeCompare(b.policy["policy-type"])).map((policy) => html`
                <h2>Policy</h2>
                <table>
                    <tbody>
                        <tr>
                            <th colspan="2">Published Policy</td>
                        </tr>
                        <tr>
                            <td class="name">Policy Type</td>
                            <td>${this.renderPolicyTypeBadge(policy.policy["policy-type"])}</td>
                        </tr>
                        ${"policy-string" in policy.policy ? html`
                            <tr>
                                <td class="name">Policy String</td>
                                <td>${this.renderMultilineCell(policy.policy["policy-string"])}</td>
                            </tr>
                        ` : nothing}
                        <tr>
                            <td class="name">Policy Domain</td>
                            <td>${policy.policy["policy-domain"]}</td>
                        </tr>
                        ${"mx-host" in policy.policy ? html`
                            <tr>
                                <td class="name">MX Host</td>
                                <td>${this.renderMultilineCell(policy.policy["mx-host"])}</td>
                            </tr>
                        ` : nothing}

                        <tr>
                            <th colspan="2">Summary</td>
                        </tr>
                        <tr>
                            <td class="name">Successful Count</td>
                            <td>${policy.summary["total-successful-session-count"]}</td>
                        </tr>
                        <tr>
                            <td class="name">Failure Count</td>
                            <td>${this.renderFailureCountBadge(policy.summary["total-failure-session-count"])}</td>
                        </tr>
                    </tbody>

                    ${"failure-details" in policy ? policy["failure-details"].map((failureDetails) => html`
                        <tbody>
                            <tr>
                                <th colspan="2">Failure Details – ${this.renderFailureResultType(failureDetails["result-type"])}</td>
                            </tr>
                            <tr>
                                <td class="name">Failure Result Type</td>
                                <td>${this.renderFailureResultType(failureDetails["result-type"])}</td>
                            </tr>
                            <tr>
                                <td class="name">Sending MTA IP</td>
                                <td>
                                    ${failureDetails["sending-mta-ip"]}
                                    <button @click="${() => this.lookupIp(failureDetails["sending-mta-ip"])}" class="button sm help" title="Search DNS hostname for IP and geolocate it">DNS and Location</button>
                                    <a class="button sm help" title="Look up WHOIS record for IP and show in new tab" target="blank" href="/ips/${failureDetails["sending-mta-ip"]}/whois">WHOIS</a>
                                </td>
                            </tr>
                        </tbody>
                        <tbody class="sourceip" style="${this.ipDetails[failureDetails["sending-mta-ip"]] ? "": "display:none"}">
                            <tr>
                                <td class="name">Sending MTA IP DNS</td>
                                <td>${this.renderIfDefined(this.ip2dns[failureDetails["sending-mta-ip"]])}
                                </td>
                            </tr>
                            <tr>
                                <td class="name">Sending MTA IP Country</td>
                                <td>${this.renderPropIfObjDefined(this.ip2location[failureDetails["sending-mta-ip"]], "country")}</td>
                            </tr>
                            <tr>
                                <td class="name">Sending MTA IP City</td>
                                <td>${this.renderPropIfObjDefined(this.ip2location[failureDetails["sending-mta-ip"]], "city")}</td>
                            </tr>
                            <tr>
                                <td class="name">Sending MTA IP ISP</td>
                                <td>${this.renderPropIfObjDefined(this.ip2location[failureDetails["sending-mta-ip"]], "isp")}</td>
                            </tr>
                            <tr>
                                <td class="name">Sending MTA IP AS</td>
                                <td>${this.renderPropIfObjDefined(this.ip2location[failureDetails["sending-mta-ip"]], "as")}</td>
                            </tr>
                            <tr>
                                <td class="name help" title="Known Proxy, VPN or Tor exit address?">Sending MTA IP Proxy</td>
                                <td>${this.renderPropIfObjDefined(this.ip2location[failureDetails["sending-mta-ip"]], "proxy")}</td>
                            </tr>
                            <tr>
                                <td class="name help" title="Known data center, hosting or colocated">Sending MTA IP Data Center</td>
                                <td>${this.renderPropIfObjDefined(this.ip2location[failureDetails["sending-mta-ip"]], "hosting")}</td>
                            </tr>
                            <tr>
                                <td class="name">Sending MTA IP Location</td>
                                <td>${this.ip2location[failureDetails["sending-mta-ip"]] === undefined ?
                                        html`<span class="faded">loading</span>` :
                                        this.renderLocation(this.ip2location[failureDetails["sending-mta-ip"]]?.lat, this.ip2location[failureDetails["sending-mta-ip"]]?.lon)
                                    }
                                </td>
                            </tr>
                        </tbody>
                        <tbody>
                            <tr>
                                <td class="name">Receiving MX Host</td>
                                <td>${failureDetails["receiving-mx-hostname"]}</td>
                            </tr>
                            ${"receiving-mx-helo" in failureDetails ? html`
                                <tr>
                                    <td class="name">Receiving MTA HELO</td>
                                    <td>${failureDetails["receiving-mx-helo"]}</td>
                                </tr>
                            ` : nothing}
                        </tbody>
                        ${"receiving-ip" in failureDetails ? html`
                            <tbody>
                                <tr>
                                    <td class="name">Receiving IP</td>
                                    <td>
                                        ${failureDetails["receiving-ip"]}
                                        <button @click="${() => this.lookupIp(failureDetails["receiving-ip"])}" class="button sm help" title="Search DNS hostname for IP and geolocate it">DNS and Location</button>
                                        <a class="button sm help" title="Look up WHOIS record for IP and show in new tab" target="blank" href="/ips/${failureDetails["receiving-ip"]}/whois">WHOIS</a>
                                    </td>
                                </tr>
                            </tbody>
                            <tbody class="sourceip" style="${this.ipDetails[failureDetails["receiving-ip"]] ? "": "display:none"}">
                                <tr>
                                    <td class="name">Receiving IP DNS</td>
                                    <td>${this.renderIfDefined(this.ip2dns[failureDetails["receiving-ip"]])}
                                    </td>
                                </tr>
                                <tr>
                                    <td class="name">Receiving IP Country</td>
                                    <td>${this.renderPropIfObjDefined(this.ip2location[failureDetails["receiving-ip"]], "country")}</td>
                                </tr>
                                <tr>
                                    <td class="name">Receiving IP City</td>
                                    <td>${this.renderPropIfObjDefined(this.ip2location[failureDetails["receiving-ip"]], "city")}</td>
                                </tr>
                                <tr>
                                    <td class="name">Receiving IP ISP</td>
                                    <td>${this.renderPropIfObjDefined(this.ip2location[failureDetails["receiving-ip"]], "isp")}</td>
                                </tr>
                                <tr>
                                    <td class="name">Receiving IP AS</td>
                                    <td>${this.renderPropIfObjDefined(this.ip2location[failureDetails["receiving-ip"]], "as")}</td>
                                </tr>
                                <tr>
                                    <td class="name help" title="Known Proxy, VPN or Tor exit address?">Receiving IP Proxy</td>
                                    <td>${this.renderPropIfObjDefined(this.ip2location[failureDetails["receiving-ip"]], "proxy")}</td>
                                </tr>
                                <tr>
                                    <td class="name help" title="Known data center, hosting or colocated">Receiving IP Data Center</td>
                                    <td>${this.renderPropIfObjDefined(this.ip2location[failureDetails["receiving-ip"]], "hosting")}</td>
                                </tr>
                                <tr>
                                    <td class="name">Receiving IP Location</td>
                                    <td>${this.ip2location[failureDetails["receiving-ip"]] === undefined ?
                                            html`<span class="faded">loading</span>` :
                                            this.renderLocation(this.ip2location[failureDetails["receiving-ip"]]?.lat, this.ip2location[failureDetails["receiving-ip"]]?.lon)
                                        }
                                    </td>
                                </tr>
                            </tbody>
                        ` : nothing}
                        <tbody>
                            <tr>
                                <td class="name">Failed Session Count</td>
                                <td>${failureDetails["failed-session-count"]}</td>
                            </tr>
                            ${"additional-information" in failureDetails ? html`
                                <tr>
                                    <td class="name">Additional Information</td>
                                    <td>${failureDetails["additional-information"]}</td>
                                </tr>
                            ` : nothing}
                            ${"failure-reason-code" in failureDetails ? html`
                                <tr>
                                    <td class="name">Failure Reason Code</td>
                                    <td>${failureDetails["failure-reason-code"]}</td>
                                </tr>
                            ` : nothing}
                        </tbody>
                    `) : nothing}
                </table>
            `)}
        `;
    }
}

customElements.define("drv-tlsrpt-report", TlsRptReport);
