/**
 * Anki UI automation using xdotool, wmctrl, and scrot.
 */
import { $ } from "bun";

const DISPLAY = process.env.DISPLAY || ":99";

export async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function waitForWindow(
  windowName: string,
  timeoutMs = 30000
): Promise<boolean> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    const result = await $`xdotool search --name ${windowName}`.quiet().nothrow();
    if (result.exitCode === 0 && result.stdout.toString().trim()) {
      return true;
    }
    await sleep(500);
  }
  return false;
}

export async function getWindowId(windowName: string): Promise<string | null> {
  const result = await $`xdotool search --name ${windowName}`.quiet().nothrow();
  if (result.exitCode === 0 && result.stdout.toString().trim()) {
    return result.stdout.toString().trim().split("\n")[0];
  }
  return null;
}

export async function focusWindow(windowId: string): Promise<boolean> {
  const result = await $`xdotool windowactivate --sync ${windowId}`.quiet().nothrow();
  await sleep(300);
  return result.exitCode === 0;
}

export async function sendKeys(
  keys: string,
  windowId?: string
): Promise<void> {
  if (windowId) {
    // Send keys directly to the window
    await $`xdotool key --window ${windowId} ${keys}`.quiet().nothrow();
  } else {
    await $`xdotool key ${keys}`.quiet().nothrow();
  }
  await sleep(500);
}

export async function typeText(text: string): Promise<void> {
  await $`xdotool type --clearmodifiers ${text}`.quiet().nothrow();
  await sleep(200);
}

export async function takeScreenshot(
  outputPath: string,
  windowId?: string
): Promise<boolean> {
  // Ensure parent directory exists
  const dir = outputPath.substring(0, outputPath.lastIndexOf("/"));
  await $`mkdir -p ${dir}`.quiet().nothrow();

  if (windowId) {
    await focusWindow(windowId);
    await sleep(200);
    // Capture focused window
    const result = await $`scrot -u -o ${outputPath}`.quiet().nothrow();
    return result.exitCode === 0;
  } else {
    // Capture entire screen
    const result = await $`scrot -o ${outputPath}`.quiet().nothrow();
    return result.exitCode === 0;
  }
}

export interface AnkiController {
  ankiProcess: ReturnType<typeof Bun.spawn> | null;
  mainWindowId: string | null;
  browseWindowId: string | null;
  collectionPath: string | null;
}

export function createAnkiController(
  collectionPath?: string
): AnkiController {
  return {
    ankiProcess: null,
    mainWindowId: null,
    browseWindowId: null,
    collectionPath: collectionPath || null,
  };
}

export async function startAnki(
  controller: AnkiController
): Promise<boolean> {
  const args = ["anki"];
  if (controller.collectionPath) {
    const baseDir = controller.collectionPath.substring(
      0,
      controller.collectionPath.lastIndexOf("/")
    );
    args.push("-b", baseDir);
  }

  console.log(`  Starting Anki: ${args.join(" ")}`);

  controller.ankiProcess = Bun.spawn(args, {
    env: { ...process.env, DISPLAY },
    stdout: "pipe",
    stderr: "pipe",
  });

  // Wait for main window
  if (!(await waitForWindow("Anki", 60000))) {
    console.error("  ERROR: Anki main window did not appear");
    return false;
  }

  await sleep(2000); // Give Anki time to fully initialize
  controller.mainWindowId = await getWindowId("Anki");
  return controller.mainWindowId !== null;
}

export async function openBrowse(
  controller: AnkiController
): Promise<boolean> {
  if (!controller.mainWindowId) {
    return false;
  }

  // Try multiple times to open Browse
  for (let attempt = 0; attempt < 3; attempt++) {
    await focusWindow(controller.mainWindowId);
    await sleep(500);

    // Ctrl+B opens Browse - send it multiple ways for robustness
    await $`xdotool key --window ${controller.mainWindowId} ctrl+b`.quiet().nothrow();
    await sleep(500);
    
    // Also try sending to focused window
    await $`xdotool key ctrl+b`.quiet().nothrow();
    await sleep(1000);

    // Wait for Browse window
    if (await waitForWindow("Browse", 5000)) {
      await sleep(1000); // Let it fully render
      controller.browseWindowId = await getWindowId("Browse");
      if (controller.browseWindowId) {
        return true;
      }
    }
    
    console.log(`  Attempt ${attempt + 1} failed, retrying...`);
  }
  
  console.error("  ERROR: Browse window did not appear after 3 attempts");
  return false;
}

export async function selectDeckInBrowse(
  controller: AnkiController,
  deckName: string
): Promise<void> {
  if (!controller.browseWindowId) {
    return;
  }

  await focusWindow(controller.browseWindowId);
  await sleep(200);

  // Select all in search box and type deck filter
  await sendKeys("ctrl+a");
  await sleep(100);
  await typeText(`deck:"${deckName}"`);
  await sleep(200);
  await sendKeys("Return");
  await sleep(500);
}

export async function screenshotBrowse(
  controller: AnkiController,
  outputPath: string
): Promise<boolean> {
  if (!controller.browseWindowId) {
    console.warn("  WARNING: No Browse window to screenshot");
    return false;
  }
  return takeScreenshot(outputPath, controller.browseWindowId);
}

export async function screenshotMain(
  controller: AnkiController,
  outputPath: string
): Promise<boolean> {
  if (!controller.mainWindowId) {
    console.warn("  WARNING: No main window to screenshot");
    return false;
  }
  return takeScreenshot(outputPath, controller.mainWindowId);
}

export async function closeBrowse(
  controller: AnkiController
): Promise<void> {
  if (controller.browseWindowId) {
    await focusWindow(controller.browseWindowId);
    await sendKeys("ctrl+w");
    await sleep(300);
    controller.browseWindowId = null;
  }
}

export async function stopAnki(controller: AnkiController): Promise<void> {
  if (controller.mainWindowId) {
    await focusWindow(controller.mainWindowId);
    await sendKeys("ctrl+q");
    await sleep(1000);
  }

  if (controller.ankiProcess) {
    controller.ankiProcess.kill();
    controller.ankiProcess = null;
  }

  controller.mainWindowId = null;
  controller.browseWindowId = null;
}

// Test function
export async function testAutomation(): Promise<boolean> {
  console.log("Testing Anki automation...");

  const controller = createAnkiController();

  try {
    console.log("Starting Anki...");
    if (!(await startAnki(controller))) {
      console.log("FAILED: Could not start Anki");
      return false;
    }

    console.log("Taking main window screenshot...");
    if (!(await screenshotMain(controller, "/tmp/test_main.png"))) {
      console.log("FAILED: Could not screenshot main window");
      return false;
    }

    console.log("Opening Browse...");
    if (!(await openBrowse(controller))) {
      console.log("FAILED: Could not open Browse");
      return false;
    }

    console.log("Taking Browse screenshot...");
    if (!(await screenshotBrowse(controller, "/tmp/test_browse.png"))) {
      console.log("FAILED: Could not screenshot Browse");
      return false;
    }

    console.log("Closing Anki...");
  } finally {
    await stopAnki(controller);
  }

  console.log("SUCCESS: All automation tests passed");
  console.log("Screenshots saved to /tmp/test_main.png and /tmp/test_browse.png");
  return true;
}

if (import.meta.main) {
  const success = await testAutomation();
  process.exit(success ? 0 : 1);
}
