import { Document, Index, SchemaBuilder } from '../index'

export interface TestDoc {
  title?: string[]
  body?: string[]
  id?: number[]
  rating?: number[]
  is_good?: boolean[]
  order?: number[]
  text?: string[]
  date?: string[]
  ip_addr?: string[]
}

// Schema builders
export const schema = () => {
  return new SchemaBuilder().addTextField('title', { stored: true }).addTextField('body').build()
}

export const schemaNumericFields = () => {
  return new SchemaBuilder()
    .addIntegerField('id', { stored: true, indexed: true, fast: true })
    .addFloatField('rating', { stored: true, indexed: true, fast: true })
    .addBooleanField('is_good', { stored: true, indexed: true })
    .addTextField('body', { stored: true, fast: true })
    .build()
}

export const schemaWithDateField = () => {
  return new SchemaBuilder()
    .addIntegerField('id', { stored: true, indexed: true })
    .addFloatField('rating', { stored: true, indexed: true })
    .addDateField('date', { stored: true, indexed: true })
    .build()
}

export const schemaWithIpAddrField = () => {
  return new SchemaBuilder()
    .addIntegerField('id', { stored: true, indexed: true })
    .addFloatField('rating', { stored: true, indexed: true })
    .addIpAddrField('ip_addr', { stored: true, indexed: true })
    .build()
}

export const spanishSchema = () => {
  return new SchemaBuilder()
    .addTextField('title', { stored: true, tokenizerName: 'es_stem' })
    .addTextField('body', { tokenizerName: 'es_stem' })
    .build()
}

// Index creators
export const createIndex = (dir?: string) => {
  const index = new Index(schema(), dir)
  const writer = index.writer(15_000_000, 1)

  // Document 1
  const doc1 = new Document()
  doc1.addText('title', 'The Old Man and the Sea')
  doc1.addText(
    'body',
    'He was an old man who fished alone in a skiff in the Gulf Stream and he had gone eighty-four days now without taking a fish.',
  )
  writer.addDocument(doc1)

  // Document 2
  const doc2 = Document.fromDict(
    {
      title: 'Of Mice and Men',
      body: "A few miles south of Soledad, the Salinas River drops in close to the hillside bank and runs deep and green. The water is warm too, for it has slipped twinkling over the yellow sands in the sunlight before reaching the narrow pool. On one side of the river the golden foothill slopes curve up to the strong and rocky Gabilan Mountains, but on the valley side the water is lined with trees—willows fresh and green with every spring, carrying in their lower leaf junctures the debris of the winter's flooding; and sycamores with mottled, white, recumbent limbs and branches that arch over the pool",
    },
    schema(),
  )
  writer.addDocument(doc2)

  // Document 3
  writer.addJson(
    JSON.stringify({
      title: ['Frankenstein', 'The Modern Prometheus'],
      body: 'You will rejoice to hear that no disaster has accompanied the commencement of an enterprise which you have regarded with such evil forebodings. I arrived here yesterday, and my first task is to assure my dear sister of my welfare and increasing confidence in the success of my undertaking.',
    }),
  )

  writer.commit()
  writer.waitMergingThreads()
  index.reload()
  return index
}

export const createIndexWithNumericFields = (dir?: string) => {
  const index = new Index(schemaNumericFields(), dir)
  const writer = index.writer(15_000_000, 1)

  const doc1 = new Document()
  doc1.addInteger('id', 1)
  doc1.addFloat('rating', 3.5)
  doc1.addBoolean('is_good', true)
  doc1.addText(
    'body',
    'He was an old man who fished alone in a skiff in the Gulf Stream and he had gone eighty-four days now without taking a fish.',
  )
  writer.addDocument(doc1)

  const doc2 = Document.fromDict(
    {
      id: 2,
      rating: 4.5,
      is_good: false,
      body: "A few miles south of Soledad, the Salinas River drops in close to the hillside bank and runs deep and green. The water is warm too, for it has slipped twinkling over the yellow sands in the sunlight before reaching the narrow pool. On one side of the river the golden foothill slopes curve up to the strong and rocky Gabilan Mountains, but on the valley side the water is lined with trees—willows fresh and green with every spring, carrying in their lower leaf junctures the debris of the winter's flooding; and sycamores with mottled, white, recumbent limbs and branches that arch over the pool",
    },
    schemaNumericFields(),
  )
  writer.addDocument(doc2)

  writer.commit()
  writer.waitMergingThreads()
  index.reload()
  return index
}

export const createIndexWithDateField = (dir?: string) => {
  const index = new Index(schemaWithDateField(), dir)
  const writer = index.writer(15_000_000, 1)

  const doc1 = new Document()
  doc1.addInteger('id', 1)
  doc1.addFloat('rating', 3.5)
  doc1.addDate('date', new Date('2021-01-01').toISOString())
  writer.addDocument(doc1)

  const doc2 = Document.fromDict(
    {
      id: 2,
      rating: 4.5,
      date: new Date('2021-01-02').toISOString(),
    },
    schemaWithDateField(),
  )
  writer.addDocument(doc2)

  writer.commit()
  writer.waitMergingThreads()
  index.reload()
  return index
}

export const createIndexWithIpAddrField = (dir?: string) => {
  const indexSchema = schemaWithIpAddrField()
  const index = new Index(indexSchema, dir)
  const writer = index.writer(15_000_000, 1)

  const doc1 = new Document()
  doc1.addInteger('id', 1)
  doc1.addFloat('rating', 3.5)
  doc1.addIpAddr('ip_addr', '10.0.0.1')
  writer.addDocument(doc1)

  const doc2 = Document.fromDict(
    {
      id: 2,
      rating: 4.5,
      ip_addr: '127.0.0.1',
    },
    indexSchema,
  )
  writer.addDocument(doc2)

  const doc3 = Document.fromDict(
    {
      id: 3,
      rating: 4.5,
      ip_addr: '::1',
    },
    indexSchema,
  )
  writer.addDocument(doc3)

  writer.commit()
  writer.waitMergingThreads()
  index.reload()
  return index
}

export const createSpanishIndex = () => {
  const index = new Index(spanishSchema())
  const writer = index.writer()

  const doc1 = new Document()
  doc1.addText('title', 'El viejo y el mar')
  doc1.addText(
    'body',
    'Era un viejo que pescaba solo en un bote en el Gulf Stream y hacía ochenta y cuatro días que no cogía un pez.',
  )
  writer.addDocument(doc1)

  const doc2 = Document.fromDict(
    {
      title: 'De ratones y hombres',
      body: 'Unas millas al sur de Soledad, el río Salinas se ahonda junto al margen de la ladera y fluye profundo y verde. Es tibia el agua, porque se ha deslizado chispeante sobre la arena amarilla y al calor del sol antes de llegar a la angosta laguna. A un lado del río, la dorada falda de la ladera se curva hacia arriba trepando hasta las montañas Gabilán, fuertes y rocosas, pero del lado del valle los árboles bordean la orilla: sauces frescos y verdes cada primavera, que en la s junturas más bajas de sus hojas muestran las consecuencias de la crecida invernal; y sicomoros de troncos veteados, blancos, recostados, y ramas quesear quean sobre el estanque',
    },
    schema(),
  )
  writer.addDocument(doc2)

  writer.addJson(
    JSON.stringify({
      title: ['Frankenstein', 'El moderno Prometeo'],
      body: 'Te alegrará saber que no ha ocurrido ningún percance al principio de una aventura que siempre consideraste cargada de malos presagios. Llegué aquí ayer, y mi primera tarea es asegurarle a mi querida hermana que me hallo perfectamente y que tengo una gran confianza en el éxito de mi empresa.',
    }),
  )

  writer.commit()
  writer.waitMergingThreads()
  index.reload()
  return index
}
