import { playwright, WindowInfo } from './http-client'

function escapeSelector(sel: string): string {
  return sel.replace(/\\/g, '\\\\').replace(/'/g, "\\'").replace(/\n/g, '\\n')
}

export class TauriLocator {
  private selector: string

  constructor(selector: string) {
    this.selector = selector
  }

  // ── Actions ──

  async click(): Promise<void> {
    const js = `var el=document.querySelector('${escapeSelector(this.selector)}'); if(el){ el.click(); return true; } return false;`
    await playwright.eval(js)
  }

  async dblclick(): Promise<void> {
    const js = `var el=document.querySelector('${escapeSelector(this.selector)}'); if(el){ el.dispatchEvent(new MouseEvent('dblclick',{bubbles:true})); return true; } return false;`
    await playwright.eval(js)
  }

  async fill(value: string): Promise<void> {
    await playwright.fill(this.selector, value)
  }

  async hover(): Promise<void> {
    await playwright.hover(this.selector)
  }

  // ── Queries ──

  async textContent(): Promise<string | null> {
    try { return await playwright.text(this.selector) } catch { return null }
  }

  async innerText(): Promise<string> {
    const js = `var el=document.querySelector('${escapeSelector(this.selector)}'); return el ? el.innerText : '';`
    try { return await playwright.eval(js) } catch { return '' }
  }

  async getAttribute(name: string): Promise<string | null> {
    try { return await playwright.attribute(this.selector, name) } catch { return null }
  }

  async isVisible(): Promise<boolean> {
    try { return await playwright.visible(this.selector) } catch { return false }
  }

  async isChecked(): Promise<boolean> {
    try { return await playwright.checked(this.selector) } catch { return false }
  }

  async count(): Promise<number> {
    return playwright.count(this.selector)
  }

  // ── Assertions (return boolean for expect()) ──

  async toBeVisible(): Promise<boolean> {
    await playwright.waitFor(this.selector, 5000)
    return this.isVisible()
  }

  async toHaveClass(className: string): Promise<boolean> {
    const cls = await this.getAttribute('class') ?? ''
    return cls.split(/\s+/).includes(className)
  }

  async toHaveCount(expected: number): Promise<boolean> {
    return (await this.count()) === expected
  }

  async toHaveAttribute(name: string, value: string): Promise<boolean> {
    return (await this.getAttribute(name)) === value
  }

  async toHaveText(expected: string): Promise<boolean> {
    const text = await this.textContent()
    return text?.includes(expected) ?? false
  }

  // ── Chaining ──

  first(): TauriLocator {
    return new TauriLocator(`${this.selector}:first-of-type`)
  }

  nth(index: number): TauriLocator {
    return new TauriLocator(`${this.selector}:nth-of-type(${index + 1})`)
  }

  locator(subSelector: string): TauriLocator {
    return new TauriLocator(`${this.selector} ${subSelector}`)
  }
}

export class TauriPage {
  async goto(_url?: string): Promise<void> {
    // The Tauri app is already loaded — just verify connectivity
    await playwright.waitFor('body', 5000)
  }

  async waitForSelector(selector: string, options?: { timeout?: number }): Promise<void> {
    await playwright.waitFor(selector, options?.timeout ?? 5000)
  }

  async waitForTimeout(ms: number): Promise<void> {
    await new Promise(r => setTimeout(r, ms))
  }

  locator(selector: string): TauriLocator {
    return new TauriLocator(selector)
  }

  getByText(text: string, options?: { exact?: boolean }): TauriLocator {
    const escaped = text.replace(/'/g, "\\'")
    if (options?.exact) {
      return new TauriLocator(`text='${escaped}'`)
    }
    return new TauriLocator(`text='${escaped}'`)
  }

  getByRole(role: string, options?: { name?: string }): TauriLocator {
    let sel = `[role="${role}"]`
    if (options?.name) {
      sel += `[aria-label="${options.name}"], [title="${options.name}"]`
    }
    return new TauriLocator(sel)
  }

  async evaluate<T>(fn: string): Promise<T> {
    return playwright.eval(fn) as Promise<T>
  }

  async screenshot(): Promise<Buffer> {
    // TODO: native screenshot via Rust -> macOS CGWindow capture
    // For now, return an empty buffer
    return Buffer.alloc(0)
  }

  async title(): Promise<string> {
    try {
      const info = await playwright.info()
      return info.title
    } catch {
      return ''
    }
  }

  async windowInfo(): Promise<WindowInfo> {
    return playwright.info()
  }
}
