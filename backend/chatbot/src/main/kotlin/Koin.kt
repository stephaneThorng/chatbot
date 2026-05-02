package dev.stephyu

import dev.stephyu.core.chat.adapter.out.memory.InMemoryConversationSessionRepository
import dev.stephyu.core.chat.adapter.out.memory.InMemoryReservationInventoryRepository
import dev.stephyu.core.chat.adapter.out.memory.InMemoryRestaurantKnowledgeRepository
import dev.stephyu.core.chat.adapter.out.nlp.HttpNlpAnalyzer
import dev.stephyu.core.chat.adapter.out.nlp.NlpClientConfig
import dev.stephyu.core.chat.application.orchestration.ChatMessageOrchestrator
import dev.stephyu.core.chat.application.port.out.ConversationSessionRepository
import dev.stephyu.core.chat.application.port.out.NlpAnalyzer
import dev.stephyu.core.chat.application.port.out.ReservationInventoryRepository
import dev.stephyu.core.chat.application.port.out.RestaurantKnowledgeRepository
import dev.stephyu.core.chat.application.service.ConversationActPreprocessor
import dev.stephyu.core.chat.application.service.IntentResolver
import dev.stephyu.core.chat.application.service.ReplyComposer
import dev.stephyu.core.chat.application.service.ReservationWorkflowService
import dev.stephyu.core.chat.application.state.ChatStateMachine
import dev.stephyu.core.chat.application.state.IdleStateHandler
import dev.stephyu.core.chat.application.state.WorkflowStateHandler
import dev.stephyu.core.chat.application.usecase.HandleChatMessageUseCase
import io.ktor.server.application.*
import java.time.Clock
import org.koin.dsl.module
import org.koin.ktor.plugin.Koin
import org.koin.logger.slf4jLogger

fun Application.configureKoin() {
    val nlpBaseUrl = environment.config.propertyOrNull("nlp.apiUrl")?.getString()
        ?: "http://localhost:8000"

    install(Koin) {
        slf4jLogger()
        modules(module {
            single { Clock.systemUTC() }
            single { NlpClientConfig(baseUrl = nlpBaseUrl) }
            single<ConversationSessionRepository> { InMemoryConversationSessionRepository() }
            single<RestaurantKnowledgeRepository> { InMemoryRestaurantKnowledgeRepository() }
            single<ReservationInventoryRepository> { InMemoryReservationInventoryRepository() }
            single<NlpAnalyzer> { HttpNlpAnalyzer(get()) }
            single { ConversationActPreprocessor() }
            single { IntentResolver() }
            single { ReplyComposer(get()) }
            single { ReservationWorkflowService(get(), get(), get()) }
            single { IdleStateHandler(get(), get()) }
            single { WorkflowStateHandler(get(), get()) }
            single { ChatStateMachine(get(), get()) }
            single { ChatMessageOrchestrator(get(), get(), get(), get(), get()) }
            single { HandleChatMessageUseCase(get(), get(), get()) }
            single<HelloService> {
                HelloService {
                    println(environment.log.info("Hello, World!"))
                }
            }
        })
    }
}
