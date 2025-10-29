const { invoke } = window.__TAURI__.tauri;
const { appWindow } = window.__TAURI__.window;

const fileListEl = document.getElementById("file-list");
const clearListBtn = document.getElementById("clear-list-btn");

async function refreshFileList() {
  fileListEl.innerHTML = "";
  const files = await invoke("get_copy_list");

  if (files.length === 0) {
    const placeholder = document.createElement("li");
    placeholder.className = "placeholder";
    placeholder.textContent = "No files copied yet.";
    fileListEl.appendChild(placeholder);
  } else {
    files.forEach((filePath) => {
      const listItem = document.createElement("li");

      const text = document.createElement("span");
      text.textContent = filePath;
      listItem.appendChild(text);

      const deleteButton = document.createElement("button");
      deleteButton.className = "delete-btn";
      deleteButton.textContent = "Delete";
      deleteButton.onclick = async (e) => {
        e.stopPropagation();
        await invoke("remove_from_copy_list", { path: filePath });
        refreshFileList();
      };

      listItem.appendChild(deleteButton);
      fileListEl.appendChild(listItem);
    });
  }
}

clearListBtn.addEventListener("click", async () => {
  await invoke("clear_copy_list");
  refreshFileList();
});

// Refresh the list when the window gets focus, which happens when it's shown.
appWindow.onFocusChanged(({ payload: focused }) => {
    if (focused) {
        refreshFileList();
    }
});

// Initial load
window.addEventListener("DOMContentLoaded", () => {
  refreshFileList();
});
