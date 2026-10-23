export function join(elements, joiner) {
    return elements.flatMap(x => [joiner, x]).slice(1);
}

// Opens a date-range dialog and resolves with the selected dates, or undefined if cancelled.
export function customDateRange(before, after) {
    return new Promise((resolve) => {
        const dialog = document.createElement("dialog");
        const form = document.createElement("form");
        form.innerHTML = `
            <h2>Custom date range</h2>
            <label>After: <input name="after" type="datetime-local"></label>
            <label>Before: <input name="before" type="datetime-local"></label>
            <button type="button" data-action="clear">Clear dates</button>
            <button type="button" data-action="cancel">Cancel</button>
            <button type="submit">Apply</button>
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

        dialog.append(form);
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
        form.querySelector('[data-action="clear"]').addEventListener("click", () => {
            afterInput.value = "";
            beforeInput.value = "";
        });
        form.querySelector('[data-action="cancel"]').addEventListener("click", () => finish(undefined));
        dialog.addEventListener("cancel", (event) => {
            event.preventDefault();
            finish(undefined);
        });

        dialog.showModal();
    });
}
