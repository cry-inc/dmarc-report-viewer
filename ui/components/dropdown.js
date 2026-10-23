import { LitElement, html, css } from "lit";
import { globalStyle } from "../style.js";

export class Dropdown extends LitElement {
    static properties = {
        title: { type: String },
        items: { type: Array },
        onChange: { attribute: false }, // Javascript callback, not an HTML attribute!
        open: { type: Boolean, state: true }, // Internal state
    };

    static styles = [globalStyle, css`
        :host {
            display: inline-block;
            position: relative;
        }

        .dropdown-button {
            display: grid;
            text-align: left;
        }

        .button-label, .button-size {
            grid-area: 1 / 1;
            white-space: nowrap;
        }

        .button-size {
            visibility: hidden;
        }

        .items {
            position: absolute;
            z-index: 1;
            top: 100%;
            left: 0;
            min-width: 100%;
            padding: 4px 0;
            background: white;
            border: 1px solid #dee2e6;
            border-radius: 3px;
            box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
        }

        .items button {
            display: block;
            width: 100%;
            padding: 5px 10px;
            border: none;
            font: inherit;
            color: #212529;
            text-align: left;
            white-space: nowrap;
            background: white;
            border-radius: 0;
        }

        .items button:hover {
            background: #f4f4f4;
        }
    `];

    constructor() {
        super();
        this.title = "";
        this.items = [];
        this.open = false;
        this.onChange = null;
        this.handleOutsideClick = this.handleOutsideClick.bind(this);
    }

    disconnectedCallback() {
        super.disconnectedCallback();
        document.removeEventListener("pointerdown", this.handleOutsideClick);
    }

    get selectedItem() {
        return this.items.find((item) => item.selected) || this.items[0];
    }

    toggle() {
        this.open = !this.open;
        if (this.open) {
            document.addEventListener("pointerdown", this.handleOutsideClick);
        } else {
            document.removeEventListener("pointerdown", this.handleOutsideClick);
        }
    }

    handleOutsideClick(event) {
        if (!event.composedPath().includes(this)) {
            this.open = false;
            document.removeEventListener("pointerdown", this.handleOutsideClick);
        }
    }

    async select(item) {
        this.open = false;
        document.removeEventListener("pointerdown", this.handleOutsideClick);
        const accepted = !this.onChange || await this.onChange(item.value) !== false;
        if (accepted) {
            this.items = this.items.map((availableItem) => ({
                value: availableItem.value,
                displayName: availableItem.displayName,
                selected: availableItem === item,
            }));
        }
    }

    render() {
        const selectedItem = this.selectedItem;
        const longestItemName = this.items.reduce((longest, item) => item.displayName.length > longest.length ? item.displayName : longest, "");
        return html`
            <button class="button dropdown-button" @click="${this.toggle}">
                <span class="button-label">${this.title}: ${selectedItem?.displayName || ""}</span>
                <span class="button-size">${this.title}: ${longestItemName}</span>
            </button>
            ${this.open ? html`
                <div class="items" role="listbox">
                    ${this.items.map((item) => html`
                        <button @click="${() => this.select(item)}">
                            ${item.displayName}
                        </button>
                    `)}
                </div>
            ` : ""}
        `;
    }
}

customElements.define("drv-dropdown", Dropdown);
