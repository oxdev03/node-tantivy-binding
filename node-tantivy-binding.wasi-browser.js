import {
  createOnMessage as __wasmCreateOnMessageForFsProxy,
  getDefaultContext as __emnapiGetDefaultContext,
  instantiateNapiModuleSync as __emnapiInstantiateNapiModuleSync,
  WASI as __WASI,
} from '@napi-rs/wasm-runtime'



const __wasi = new __WASI({
  version: 'preview1',
})

const __wasmUrl = new URL('./node-tantivy-binding.wasm32-wasi.wasm', import.meta.url).href
const __emnapiContext = __emnapiGetDefaultContext()


const __sharedMemory = new WebAssembly.Memory({
  initial: 4000,
  maximum: 65536,
  shared: true,
})

const __wasmFile = await fetch(__wasmUrl).then((res) => res.arrayBuffer())

const {
  instance: __napiInstance,
  module: __wasiModule,
  napiModule: __napiModule,
} = __emnapiInstantiateNapiModuleSync(__wasmFile, {
  context: __emnapiContext,
  asyncWorkPoolSize: 4,
  wasi: __wasi,
  onCreateWorker() {
    const worker = new Worker(new URL('./wasi-worker-browser.mjs', import.meta.url), {
      type: 'module',
    })


    return worker
  },
  overwriteImports(importObject) {
    importObject.env = {
      ...importObject.env,
      ...importObject.napi,
      ...importObject.emnapi,
      memory: __sharedMemory,
    }
    return importObject
  },
  beforeInit({ instance }) {
    for (const name of Object.keys(instance.exports)) {
      if (name.startsWith('__napi_register__')) {
        instance.exports[name]()
      }
    }
  },
})
export default __napiModule.exports
export const AllButQueryForbiddenError = __napiModule.exports.AllButQueryForbiddenError
export const DateFormatError = __napiModule.exports.DateFormatError
export const Document = __napiModule.exports.Document
export const ExpectedBase64Error = __napiModule.exports.ExpectedBase64Error
export const ExpectedBoolError = __napiModule.exports.ExpectedBoolError
export const ExpectedFloatError = __napiModule.exports.ExpectedFloatError
export const ExpectedIntError = __napiModule.exports.ExpectedIntError
export const Explanation = __napiModule.exports.Explanation
export const Facet = __napiModule.exports.Facet
export const FacetFormatError = __napiModule.exports.FacetFormatError
export const FieldDoesNotExistError = __napiModule.exports.FieldDoesNotExistError
export const FieldDoesNotHavePositionsIndexedError = __napiModule.exports.FieldDoesNotHavePositionsIndexedError
export const FieldNotIndexedError = __napiModule.exports.FieldNotIndexedError
export const Filter = __napiModule.exports.Filter
export const FilterStatic = __napiModule.exports.FilterStatic
export const Index = __napiModule.exports.Index
export const IndexWriter = __napiModule.exports.IndexWriter
export const IpFormatError = __napiModule.exports.IpFormatError
export const NoDefaultFieldDeclaredError = __napiModule.exports.NoDefaultFieldDeclaredError
export const PhrasePrefixRequiresAtLeastTwoTermsError = __napiModule.exports.PhrasePrefixRequiresAtLeastTwoTermsError
export const Query = __napiModule.exports.Query
export const RangeMustNotHavePhraseError = __napiModule.exports.RangeMustNotHavePhraseError
export const Schema = __napiModule.exports.Schema
export const SchemaBuilder = __napiModule.exports.SchemaBuilder
export const Searcher = __napiModule.exports.Searcher
export const Snippet = __napiModule.exports.Snippet
export const SnippetGenerator = __napiModule.exports.SnippetGenerator
export const SyntaxError = __napiModule.exports.SyntaxError
export const TextAnalyzer = __napiModule.exports.TextAnalyzer
export const TextAnalyzerBuilder = __napiModule.exports.TextAnalyzerBuilder
export const Tokenizer = __napiModule.exports.Tokenizer
export const TokenizerStatic = __napiModule.exports.TokenizerStatic
export const UnknownTokenizerError = __napiModule.exports.UnknownTokenizerError
export const UnsupportedQueryError = __napiModule.exports.UnsupportedQueryError
export const FieldType = __napiModule.exports.FieldType
export const getVersion = __napiModule.exports.getVersion
export const Occur = __napiModule.exports.Occur
export const Order = __napiModule.exports.Order
