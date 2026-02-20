const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;


let fileListEl;
let clearListBtn;

async function refreshFileList() {
  if (!fileListEl) return;
  
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
        // The event will trigger refreshFileList(), but we can also call it directly for immediate feedback
        await refreshFileList();
      };

      listItem.appendChild(deleteButton);
      fileListEl.appendChild(listItem);
    });
  }
}

// Initialize when DOM is ready
window.addEventListener("DOMContentLoaded", async () => {
  fileListEl = document.getElementById("file-list");
  clearListBtn = document.getElementById("clear-list-btn");

  if (!fileListEl || !clearListBtn) {
    console.error("Required DOM elements not found");
    return;
  }

  // Set up event listener for backend updates
  try {
    await listen("list_updated", () => {
      refreshFileList();
    });
    console.log("Event listener for 'list_updated' registered successfully");
  } catch (error) {
    console.error("Failed to register event listener:", error);
  }

  // Set up clear button
  clearListBtn.addEventListener("click", async () => {
    await invoke("clear_copy_list");
    // The event will trigger refreshFileList(), but we can also call it directly for immediate feedback
    await refreshFileList();
  });

  // Initial load
  await refreshFileList();
});
