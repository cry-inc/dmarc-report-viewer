export function join(elements, joiner) {
    return elements.flatMap(x => [joiner, x]).slice(1);
}

// Opens a date-range dialog and resolves with the selected dates, or undefined if cancelled.
export function customDateRange(before, after) {
    return new Promise((resolve) => {
        const dialog = document.createElement("dialog");
        const style = document.createElement("style");
        style.textContent = `
            dialog {
                border: none;
                border-radius: 5px;
                padding: 40px;
            }

            dialog::backdrop {
                background: rgb(0 0 0 / 70%);
            }

            dialog h2 {
                margin-top: 0px;
            }

            .dialog-actions {
                display: flex;
                justify-content: flex-end;
                gap: 8px;
                margin-top: 24px;
            }
        `;
        const form = document.createElement("form");
        form.innerHTML = `
            <h2>Select Time Range</h2>
            <label>Start: <input name="after" type="datetime-local"></label>
            <label>End: <input name="before" type="datetime-local"></label>
            <div class="dialog-actions">
                <button type="button" data-action="cancel">Cancel</button>
                <button type="submit">Apply</button>
            </div>
        `;

        const toLocalInput = (date) => {
            if (!date) return "";
            const localDate = new Date(date.getTime() - date.getTimezoneOffset() * 60_000);
            return localDate.toISOString().slice(0, 16);
        };
        const afterInput = form.elements.namedItem("after");
        const beforeInput = form.elements.namedItem("before");
        afterInput.value = toLocalInput(after);
        beforeInput.value = toLocalInput(before);

        dialog.append(style, form);
        document.body.append(dialog);

        const finish = (result) => {
            dialog.close();
            dialog.remove();
            resolve(result);
        };
        form.addEventListener("submit", (event) => {
            event.preventDefault();
            finish({
                after: afterInput.value ? new Date(afterInput.value) : undefined,
                before: beforeInput.value ? new Date(beforeInput.value) : undefined,
            });
        });
        form.querySelector('[data-action="cancel"]').addEventListener("click", () => finish(undefined));
        dialog.addEventListener("cancel", (event) => {
            event.preventDefault();
            finish(undefined);
        });

        dialog.showModal();
    });
}
