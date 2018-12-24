package texlab

import org.eclipse.lsp4j.*
import org.eclipse.lsp4j.jsonrpc.messages.Either
import org.eclipse.lsp4j.services.TextDocumentService
import texlab.completion.AggregateCompletionProvider
import texlab.completion.CompletionProvider
import texlab.completion.CompletionRequest
import texlab.completion.OrderByQualityProvider
import texlab.completion.latex.*
import texlab.folding.*
import texlab.rename.AggregateRenamer
import texlab.rename.LatexEnvironmentRenamer
import texlab.rename.RenameRequest
import texlab.rename.Renamer
import java.net.URI
import java.util.concurrent.CompletableFuture

class TextDocumentServiceImpl(private val workspace: Workspace) : TextDocumentService {

    private val completionProvider: CompletionProvider =
            OrderByQualityProvider(
                    AggregateCompletionProvider(
                            LatexIncludeProvider(workspace),
                            LatexBibliographyProvider(workspace),
                            PgfLibraryProvider(),
                            TikzLibraryProvider(),
                            LatexColorProvider(),
                            DefineColorModelProvider(),
                            DefineColorSetModelProvider(),
                            LatexLabelProvider(),
                            LatexBeginCommandProvider(),
                            LatexKernelEnvironmentProvider(),
                            LatexUserEnvironmentProvider(),
                            LatexKernelCommandProvider(),
                            LatexUserCommandProvider()))

    private val renamer: Renamer = AggregateRenamer(LatexEnvironmentRenamer)

    private val foldingProvider: FoldingProvider =
            AggregateFoldingProvider(
                    LatexEnvironmentFoldingProvider,
                    LatexSectionFoldingProvider)

    companion object {
        private const val MAX_COMPLETIONS_ITEMS_COUNT = 100
    }

    override fun didOpen(params: DidOpenTextDocumentParams) {
        params.textDocument.apply {
            val language = getLanguageById(languageId) ?: return
            synchronized(workspace) {
                workspace.create(URI.create(uri), language, text)
            }
        }
    }

    override fun didChange(params: DidChangeTextDocumentParams) {
        val uri = URI.create(params.textDocument.uri)
        synchronized(workspace) {
            workspace.update(uri, params.contentChanges, params.textDocument.version)
        }
    }

    override fun didSave(params: DidSaveTextDocumentParams) {
    }

    override fun didClose(params: DidCloseTextDocumentParams) {
    }

    override fun documentSymbol(params: DocumentSymbolParams):
            CompletableFuture<MutableList<Either<SymbolInformation, DocumentSymbol>>> {
        synchronized(workspace) {
            val uri = URI.create(params.textDocument.uri)
            val symbols = workspace.documents
                    .firstOrNull { it.uri == uri }
                    ?.documentSymbol(workspace)
                    ?.map { Either.forRight<SymbolInformation, DocumentSymbol>(it) }
                    ?.toMutableList()
                    ?: mutableListOf()
            return CompletableFuture.completedFuture(symbols)
        }
    }

    override fun rename(params: RenameParams): CompletableFuture<WorkspaceEdit?> {
        synchronized(workspace) {
            val uri = URI.create(params.textDocument.uri)
            val relatedDocuments = workspace.relatedDocuments(uri)
            val request = RenameRequest(uri, relatedDocuments, params.position, params.newName)
            return CompletableFuture.completedFuture(renamer.rename(request))
        }
    }

    override fun documentLink(params: DocumentLinkParams): CompletableFuture<MutableList<DocumentLink>> {
        synchronized(workspace) {
            val uri = URI.create(params.textDocument.uri)
            val links = workspace.documents
                    .filter { it.isFile }
                    .firstOrNull { it.uri == uri }
                    ?.documentLink(workspace)
                    ?.toMutableList()
                    ?: mutableListOf()

            return CompletableFuture.completedFuture(links)
        }
    }

    override fun completion(params: CompletionParams):
            CompletableFuture<Either<MutableList<CompletionItem>, CompletionList>> {
        synchronized(workspace) {
            val uri = URI.create(params.textDocument.uri)
            val relatedDocuments = workspace.relatedDocuments(uri)
            val request = CompletionRequest(uri, relatedDocuments, params.position)
            val items = completionProvider.getItems(request).toList()
            val list = CompletionList(items.size == MAX_COMPLETIONS_ITEMS_COUNT, items)
            return CompletableFuture.completedFuture(Either.forRight(list))
        }
    }

    override fun foldingRange(params: FoldingRangeRequestParams): CompletableFuture<MutableList<FoldingRange>> {
        synchronized(workspace) {
            val uri = URI.create(params.textDocument.uri)
            val document = workspace.documents
                    .firstOrNull { it.uri == uri }
                    ?: return CompletableFuture.completedFuture(null)

            val request = FoldingRequest(document)
            val foldings = foldingProvider.fold(request).toMutableList()
            return CompletableFuture.completedFuture(foldings)
        }
    }
}
