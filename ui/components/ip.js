import { LitElement, html, css } from "lit";
import { globalStyle } from "../style.js";

class IpDialog extends LitElement {
    static properties = {
        ip: { type: String },
        dns: { state: true },
        location: { state: true },
    };

    static styles = [globalStyle, css`
        :host {
            display: block;
        }

        h2 {
            margin-top: 0;
        }

        .dialog-actions {
            display: flex;
            justify-content: center;
            gap: 8px;
            margin-top: 20px;
        }

        td.name {
            width: 100px;
        }
    `];

    constructor() {
        super();
        this.ip = "";
        this.dns = undefined;
        this.location = undefined;
    }

    async getDnsForIp(ip) {
        const response = await fetch("ips/" + ip + "/dns");
        if (response.status === 200) {
            const result = await response.text();
            this.dns = result;
        } else {
            this.dns = null;
        }
        this.requestUpdate();
    }

    async getLocationForIp(ip) {
        const response = await fetch("ips/" + ip + "/location");
        if (response.status === 200) {
            const result = await response.json();
            this.location = result;
        } else {
            this.location = null;
        }
        this.requestUpdate();
    }

    async open() {
        document.body.append(this);
        await this.updateComplete;
        this.renderRoot.querySelector("dialog").showModal();
        this.getDnsForIp(this.ip);
        this.getLocationForIp(this.ip);
    }

    close() {
        const dialog = this.renderRoot.querySelector("dialog");
        if (dialog.open) dialog.close();
        this.remove();
    }

    renderOptional(value) {
        if (value !== null && value !== undefined) {
            return html`${value}`;
        }
        return html`<span class="faded">n/a</span>`;
    }

    renderLocation() {
        if (this.location === undefined) {
            return html`<span class="faded">loading...</span>`;
        }
        if (!this.location || this.location.lat == null || this.location.lon == null) {
            return html`<span class="faded">n/a</span>`;
        }
        const { lat, lon } = this.location;
        return html`<a target="_blank" title="Show on OpenStreetMap" href="https://www.openstreetmap.org/#map=8/${lat}/${lon}">${lat}, ${lon}</a>`;
    }

    renderLocationProperty(property) {
        if (this.location === undefined) {
            return html`<span class="faded">loading...</span>`;
        }
        return this.renderOptional(this.location?.[property]);
    }

    render() {
        return html`
            <dialog @cancel=${(event) => { event.preventDefault(); this.close(); }}>
                <h2>IP Details</h2>
                <table>
                    <tbody>
                        <tr><td class="name">IP</td><td>${this.ip}</td></tr>
                        <tr><td class="name">DNS</td><td>${this.renderOptional(this.dns)}</td></tr>
                        <tr><td class="name">Country</td><td>${this.renderLocationProperty("country")}</td></tr>
                        <tr><td class="name">City</td><td>${this.renderLocationProperty("city")}</td></tr>
                        <tr><td class="name">ISP</td><td>${this.renderLocationProperty("isp")}</td></tr>
                        <tr><td class="name">AS</td><td>${this.renderLocationProperty("as")}</td></tr>
                        <tr><td class="name" title="Known Proxy, VPN or Tor exit address?">Proxy</td><td>${this.renderLocationProperty("proxy")}</td></tr>
                        <tr><td class="name" title="Known data center, hosting or colocated">Data Center</td><td>${this.renderLocationProperty("hosting")}</td></tr>
                        <tr><td class="name">Location</td><td>${this.renderLocation()}</td></tr>
                    </tbody>
                </table>
                <div class="dialog-actions">
                    <a class="button" title="Look up WHOIS record for IP and show in new tab" target="_blank" href="ips/${this.ip}/whois">Open WHOIS</a>
                    <button class="button" type="button" @click=${this.close}>Close</button>
                </div>
            </dialog>
        `;
    }
}

customElements.define("drv-ip-dialog", IpDialog);

// Shows dialog with information about the supplied IP.
export function ipDialog(ip) {
    const dialog = new IpDialog();
    dialog.ip = ip;
    return dialog.open();
}
