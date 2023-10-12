import { test, expect, webkit } from '@playwright/test'

test('basic', async ({ page }) => {
  await page.goto('http://localhost/')
  await page.locator('button:nth-child(6)').click()
  await page.locator('button:nth-child(2)').first().click()
  await page
    .getByText(
      '[ 西电 CTF 终端 ] _ 为世界上所有的美好而战 (C) 2022 - 2023 西安电子科技大学 网络与信息安全学院 | 陕ICP备05016463号 '
    )
    .click()
  await page.locator('button:nth-child(6)').click()
  await page.getByRole('button', { name: 'YES' }).click()
  await page.getByRole('link', { name: 'Wiki' }).click()
  await page.getByRole('link', { name: '软件逆向工程' }).click()
  await page.getByRole('link', { name: '编程语言' }).click()
  await page.getByRole('link', { name: '混淆方法' }).click()
  await page.getByRole('link', { name: 'Self-Modified Code' }).click()
  await page.getByRole('link', { name: 'Playground' }).click()
  await page.getByRole('link', { name: 'Games' }).click()
})

test('webkit', async () => {
  const browser = await webkit.launch()
  const page = await browser.newPage()
  await page.goto('http://localhost/')
  await page.locator('button:nth-child(6)').click()
  await page.locator('button:nth-child(2)').first().click()
  await page
    .getByText(
      '[ 西电 CTF 终端 ] _ 为世界上所有的美好而战 (C) 2022 - 2023 西安电子科技大学 网络与信息安全学院 | 陕ICP备05016463号 '
    )
    .click()
  await page.locator('button:nth-child(6)').click()
  await page.getByRole('button', { name: 'YES' }).click()
  await page.getByRole('link', { name: 'Wiki' }).click()
  await page.getByRole('link', { name: '软件逆向工程' }).click()
  await page.getByRole('link', { name: '编程语言' }).click()
  await page.getByRole('link', { name: '混淆方法' }).click()
  await page.getByRole('link', { name: 'Self-Modified Code' }).click()
  await page.getByRole('link', { name: 'Playground' }).click()
  await page.getByRole('link', { name: 'Games' }).click()
})
