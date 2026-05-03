package dev.stephyu

import dev.stephyu.core.chat.adapter.out.memory.InMemoryConversationSessionRepository
import dev.stephyu.core.chat.adapter.out.memory.InMemoryReservationInventoryRepository
import dev.stephyu.core.chat.adapter.out.memory.InMemoryRestaurantKnowledgeRepository
import dev.stephyu.core.chat.adapter.out.nlp.HttpNlpAnalyzer
import dev.stephyu.core.chat.adapter.out.nlp.NlpClientConfig
import dev.stephyu.core.chat.application.intent.handler.knowledge.ContactRequestIntentHandler
import dev.stephyu.core.chat.application.intent.handler.IntentHandler
import dev.stephyu.core.chat.application.intent.handler.knowledge.LocationRequestIntentHandler
import dev.stephyu.core.chat.application.intent.handler.knowledge.MenuRequestIntentHandler
import dev.stephyu.core.chat.application.intent.handler.knowledge.OpeningHoursIntentHandler
import dev.stephyu.core.chat.application.intent.handler.knowledge.PricingRequestIntentHandler
import dev.stephyu.core.chat.application.intent.handler.reservation.ReservationCancelIntentHandler
import dev.stephyu.core.chat.application.intent.handler.reservation.ReservationCreateIntentHandler
import dev.stephyu.core.chat.application.intent.handler.reservation.ReservationModifyIntentHandler
import dev.stephyu.core.chat.application.intent.handler.reservation.ReservationStatusIntentHandler
import dev.stephyu.core.chat.application.coordinator.ConversationCoordinator
import dev.stephyu.core.chat.application.port.out.ConversationSessionRepository
import dev.stephyu.core.chat.application.port.out.NlpAnalyzer
import dev.stephyu.core.chat.application.port.out.ReservationInventoryRepository
import dev.stephyu.core.chat.application.port.out.RestaurantKnowledgeRepository
import dev.stephyu.core.chat.application.signal.ConversationSignalExtractor
import dev.stephyu.core.chat.application.intent.decision.IntentDecisionEngine
import dev.stephyu.core.chat.application.state.ConversationStateDispatcher
import dev.stephyu.core.chat.application.state.IdleStateHandler
import dev.stephyu.core.chat.application.state.WorkflowStateHandler
import dev.stephyu.core.chat.application.usecase.HandleConversationUseCase
import dev.stephyu.core.chat.application.workflow.CancelWorkflowRule
import dev.stephyu.core.chat.application.workflow.EnterConfirmationRule
import dev.stephyu.core.chat.application.workflow.FillRequirementsRule
import dev.stephyu.core.chat.application.workflow.ResolveConfirmationRule
import dev.stephyu.core.chat.application.workflow.WorkflowEngine
import dev.stephyu.core.chat.application.intent.catalog.IntentCatalog
import io.ktor.server.application.Application
import io.ktor.server.application.install
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
            single { ConversationSignalExtractor() }

            single { CancelWorkflowRule() }
            single { FillRequirementsRule() }
            single { EnterConfirmationRule() }
            single { ResolveConfirmationRule() }
            single {
                WorkflowEngine(
                    rules = listOf(
                        get<CancelWorkflowRule>(),
                        get<FillRequirementsRule>(),
                        get<EnterConfirmationRule>(),
                        get<ResolveConfirmationRule>(),
                    ),
                    clock = get(),
                )
            }

            single { ReservationCreateIntentHandler(get(), get(), get()) }
            single { ReservationModifyIntentHandler(get(), get(), get()) }
            single { ReservationCancelIntentHandler(get()) }
            single { ReservationStatusIntentHandler() }
            single { MenuRequestIntentHandler(get()) }
            single { OpeningHoursIntentHandler(get()) }
            single { LocationRequestIntentHandler(get()) }
            single { PricingRequestIntentHandler(get()) }
            single { ContactRequestIntentHandler(get()) }

            single {
                IntentCatalog(
                    intentServices = listOf<IntentHandler>(
                        get<ReservationCreateIntentHandler>(),
                        get<ReservationModifyIntentHandler>(),
                        get<ReservationCancelIntentHandler>(),
                        get<ReservationStatusIntentHandler>(),
                        get<MenuRequestIntentHandler>(),
                        get<OpeningHoursIntentHandler>(),
                        get<LocationRequestIntentHandler>(),
                        get<PricingRequestIntentHandler>(),
                        get<ContactRequestIntentHandler>(),
                    ),
                )
            }
            single { IntentDecisionEngine(get()) }
            single { IdleStateHandler(get()) }
            single { WorkflowStateHandler(get()) }
            single { ConversationStateDispatcher(get(), get()) }
            single { ConversationCoordinator(get(), get(), get(), get()) }
            single { HandleConversationUseCase(get(), get(), get()) }
            single<HelloService> {
                HelloService {
                    println(environment.log.info("Hello, World!"))
                }
            }
        })
    }
}

