export function tableToCsv(data: Array<Array<object>>) {
  return data
    .map(
      (row) =>
        row
          .map(String) // convert every value to String
          .map((v) => v.replaceAll('"', '""')) // escape double quotes
          .map((v) => `"${v}"`) // quote it
          .join(',') // comma-separated
    )
    .join('\r\n') // rows starting on new lines
}

export function arrayObjectToCsv(data: Array<object>) {
  return tableToCsv([Object.keys(data[0]), ...data.map(Object.values)])
}
