import promises = require("node:fs/promises");

type City = string;
type Temperature = {
  minimum: number;
  maximum: number;
  sum: number;
  n: number;
};

async function main() {
  const file = process.env.FILE || "";
  const handle = await promises.open(file);

  const map: Map<City, Temperature> = new Map();

  for await (const line of handle.readLines()) {
    const [cityTemp, temperatureStr] = line.split(";");
    const city = cityTemp as string;
    const temperature = parseFloat(temperatureStr as string);

    const foundTemperature = map.get(city);
    if (foundTemperature) {
      if (temperature < foundTemperature.minimum) {
        foundTemperature.minimum = temperature;
      }

      if (temperature > foundTemperature.maximum) {
        foundTemperature.maximum = temperature;
      }

      foundTemperature.sum += temperature;
      foundTemperature.n++;
    } else {
      const newTemperature: Temperature = {
        minimum: temperature,
        maximum: temperature,
        sum: temperature,
        n: 1,
      };
      map.set(city, newTemperature);
    }
  }

  const s = format(map);
  console.log(s);
}

function format(map: Map<City, Temperature>): string {
  const stations: string[] = [];
  for (const [city, temperature] of map.entries()) {
    const mean = (temperature.sum / temperature.n).toFixed(1);
    stations.push(
      `${city}=${temperature.minimum}/${mean}/${temperature.maximum}`,
    );
  }

  stations.sort();
  const measures = stations.join(", ");
  return `{${measures}}`;
}

main().catch((e) => console.error(e));
