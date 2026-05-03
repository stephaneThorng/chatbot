package dev.stephyu

import dev.stephyu.core.chat.adapter.out.memory.InMemoryConversationSessionRepository
import dev.stephyu.core.chat.adapter.out.memory.InMemoryReservationInventoryRepository
import dev.stephyu.core.chat.adapter.out.memory.InMemoryRestaurantKnowledgeRepository
import dev.stephyu.core.chat.adapter.out.nlp.HttpNlpAnalyzer
import dev.stephyu.core.chat.adapter.out.nlp.NlpClientConfig
import dev.stephyu.core.chat.application.intent.ContactRequestIntentService
import dev.stephyu.core.chat.application.intent.IntentService
import dev.stephyu.core.chat.application.intent.LocationRequestIntentService
import dev.stephyu.core.chat.application.intent.MenuRequestIntentService
import dev.stephyu.core.chat.application.intent.OpeningHoursIntentService
import dev.stephyu.core.chat.application.intent.PricingRequestIntentService
import dev.stephyu.core.chat.application.intent.ReservationCancelIntentService
import dev.stephyu.core.chat.application.intent.ReservationCreateIntentService
import dev.stephyu.core.chat.application.intent.ReservationModifyIntentService
import dev.stephyu.core.chat.application.intent.ReservationStatusIntentService
import dev.stephyu.core.chat.application.orchestration.ConversationOrchestrator
import dev.stephyu.core.chat.application.port.out.ConversationSessionRepository
import dev.stephyu.core.chat.application.port.out.NlpAnalyzer
import dev.stephyu.core.chat.application.port.out.ReservationInventoryRepository
import dev.stephyu.core.chat.application.port.out.RestaurantKnowledgeRepository
import dev.stephyu.core.chat.application.service.ConversationActPreprocessor
import dev.stephyu.core.chat.application.service.IntentResolver
import dev.stephyu.core.chat.application.state.ConversationStateMachine
import dev.stephyu.core.chat.application.state.IdleStateHandler
import dev.stephyu.core.chat.application.state.WorkflowStateHandler
import dev.stephyu.core.chat.application.usecase.HandleConversationUseCase
import dev.stephyu.core.chat.application.workflow.CancelWorkflowRule
import dev.stephyu.core.chat.application.workflow.EnterConfirmationRule
import dev.stephyu.core.chat.application.workflow.FillRequirementsRule
import dev.stephyu.core.chat.application.workflow.ResolveConfirmationRule
import dev.stephyu.core.chat.application.workflow.WorkflowEngine
import dev.stephyu.core.chat.config.ConversationConfig
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
            single { ConversationActPreprocessor() }

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

            single { ReservationCreateIntentService(get(), get(), get()) }
            single { ReservationModifyIntentService(get(), get(), get()) }
            single { ReservationCancelIntentService(get()) }
            single { ReservationStatusIntentService() }
            single { MenuRequestIntentService(get()) }
            single { OpeningHoursIntentService(get()) }
            single { LocationRequestIntentService(get()) }
            single { PricingRequestIntentService(get()) }
            single { ContactRequestIntentService(get()) }

            single {
                ConversationConfig(
                    intentServices = listOf<IntentService>(
                        get<ReservationCreateIntentService>(),
                        get<ReservationModifyIntentService>(),
                        get<ReservationCancelIntentService>(),
                        get<ReservationStatusIntentService>(),
                        get<MenuRequestIntentService>(),
                        get<OpeningHoursIntentService>(),
                        get<LocationRequestIntentService>(),
                        get<PricingRequestIntentService>(),
                        get<ContactRequestIntentService>(),
                    ),
                )
            }
            single { IntentResolver(get()) }
            single { IdleStateHandler(get()) }
            single { WorkflowStateHandler(get()) }
            single { ConversationStateMachine(get(), get()) }
            single { ConversationOrchestrator(get(), get(), get(), get()) }
            single { HandleConversationUseCase(get(), get(), get()) }
            single<HelloService> {
                HelloService {
                    println(environment.log.info("Hello, World!"))
                }
            }
        })
    }
}
