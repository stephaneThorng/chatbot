package dev.stephyu.core.chat

import dev.stephyu.core.chat.adapter.out.memory.InMemoryConversationSessionRepository
import dev.stephyu.core.chat.adapter.out.memory.InMemoryReservationInventoryRepository
import dev.stephyu.core.chat.adapter.out.memory.InMemoryRestaurantKnowledgeRepository
import dev.stephyu.core.chat.application.command.HandleChatMessageCommand
import dev.stephyu.core.chat.application.orchestration.ChatMessageOrchestrator
import dev.stephyu.core.chat.application.port.out.NlpAnalyzer
import dev.stephyu.core.chat.application.service.ConversationActPreprocessor
import dev.stephyu.core.chat.application.service.IntentResolver
import dev.stephyu.core.chat.application.service.ReplyComposer
import dev.stephyu.core.chat.application.service.ReservationWorkflowService
import dev.stephyu.core.chat.application.state.ChatStateMachine
import dev.stephyu.core.chat.application.state.IdleStateHandler
import dev.stephyu.core.chat.application.state.WorkflowStateHandler
import dev.stephyu.core.chat.application.usecase.HandleChatMessageUseCase
import dev.stephyu.core.chat.domain.ConversationAct
import dev.stephyu.core.chat.domain.EntityType
import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.NlpAnalysis
import dev.stephyu.core.chat.domain.NlpAnalysisContext
import dev.stephyu.core.chat.domain.NlpEntity
import dev.stephyu.core.chat.domain.NlpIntent
import dev.stephyu.core.chat.domain.SlotName
import java.time.Clock
import java.time.Instant
import java.time.ZoneOffset
import kotlinx.coroutines.runBlocking
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class ChatOrchestrationTest {
    @Test
    fun `backend trusts NLP for reservation creation`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer {
            analysis(IntentName.RESERVATION_CREATE)
        })

        val result = useCase.handle(HandleChatMessageCommand("I want to make a reservation", null))

        assertEquals("reservation_create", result.intent.wireName)
        assertEquals("RESERVATION_CREATION", result.state.name)
    }

    @Test
    fun `leading greeting prefixes reply while NLP owns primary intent`() = runBlocking {
        val analyzer = ScriptedNlpAnalyzer {
            analysis(IntentName.RESERVATION_CREATE)
        }
        val useCase = useCaseWithNlp(analyzer)

        val result = useCase.handle(HandleChatMessageCommand("Hello, I want a reservation", null))

        assertEquals("reservation_create", result.intent.wireName)
        assertEquals(ConversationAct.GREETING, result.conversationAct)
        assertEquals("RESERVATION_CREATION", result.state.name)
        assertTrue(result.reply.startsWith("Hello. "))
        assertEquals(listOf("I want a reservation"), analyzer.texts)
    }

    @Test
    fun `active reservation flow continues on unknown NLP intent`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer { text, context ->
            when {
                "reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_CREATE)
                context?.requiredSlots?.contains(SlotName.DATE) == true && "July 4" in text ->
                    analysis(IntentName.UNKNOWN, entity(EntityType.DATE, "July 4"))
                context?.requiredSlots?.contains(SlotName.TIME) == true && "7:30pm" in text ->
                    analysis(IntentName.UNKNOWN, entity(EntityType.TIME, "7:30pm"))
                context?.requiredSlots?.contains(SlotName.PEOPLE) == true && text == "5" ->
                    analysis(IntentName.UNKNOWN, entity(EntityType.PEOPLE_COUNT, "5"))
                else -> analysis(IntentName.UNKNOWN)
            }
        })

        val start = useCase.handle(HandleChatMessageCommand("I need a new reservation", null))
        val name = useCase.handle(HandleChatMessageCommand("Stephane", start.sessionId))
        val date = useCase.handle(HandleChatMessageCommand("July 4", start.sessionId))
        val time = useCase.handle(HandleChatMessageCommand("7:30pm", start.sessionId))
        val people = useCase.handle(HandleChatMessageCommand("5", start.sessionId))

        assertEquals("RESERVATION_CREATION", name.state.name)
        assertEquals("July 4, 2026", date.slots[SlotName.DATE])
        assertEquals("19:30", time.slots[SlotName.TIME])
        assertEquals("RESERVATION_CREATION", people.state.name)
        assertTrue(people.reply.contains("at 19:30"))
    }

    @Test
    fun `blank message preserves active workflow context`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer { text, context ->
            when {
                "reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_CREATE)
                context?.requiredSlots?.contains(SlotName.DATE) == true && "July 4" in text ->
                    analysis(IntentName.UNKNOWN, entity(EntityType.DATE, "July 4"))
                else -> analysis(IntentName.UNKNOWN)
            }
        })

        val start = useCase.handle(HandleChatMessageCommand("I need a new reservation", null))
        val name = useCase.handle(HandleChatMessageCommand("Stephane", start.sessionId))
        val blank = useCase.handle(HandleChatMessageCommand("   ", start.sessionId))

        assertEquals("RESERVATION_CREATION", name.state.name)
        assertEquals("RESERVATION_CREATION", blank.state.name)
        assertEquals("Stephane", blank.slots[SlotName.NAME])
        assertEquals(listOf(SlotName.DATE, SlotName.TIME, SlotName.PEOPLE), blank.missingSlots)
    }

    @Test
    fun `cancel reply includes reservation details and does not trap future intents`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer { text, context ->
            when {
                "cancel" in text.lowercase() -> analysis(IntentName.RESERVATION_CANCEL)
                "opening hours" in text.lowercase() -> analysis(IntentName.OPENING_HOURS)
                "reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_CREATE)
                text == "July 4" -> analysis(IntentName.UNKNOWN, entity(EntityType.DATE, text))
                text == "7:30pm" -> analysis(IntentName.UNKNOWN, entity(EntityType.TIME, text))
                text == "5" -> analysis(IntentName.UNKNOWN, entity(EntityType.PEOPLE_COUNT, text))
                else -> analysis(IntentName.UNKNOWN)
            }
        })

        val start = useCase.handle(HandleChatMessageCommand("I need a new reservation", null))
        useCase.handle(HandleChatMessageCommand("Stephane", start.sessionId))
        useCase.handle(HandleChatMessageCommand("July 4", start.sessionId))
        useCase.handle(HandleChatMessageCommand("7:30pm", start.sessionId))
        useCase.handle(HandleChatMessageCommand("5", start.sessionId))
        useCase.handle(HandleChatMessageCommand("yes", start.sessionId))

        val cancel = useCase.handle(HandleChatMessageCommand("I want to cancel my reservation", start.sessionId))
        val confirmedCancel = useCase.handle(HandleChatMessageCommand("yes", start.sessionId))
        val openingHours = useCase.handle(HandleChatMessageCommand("opening hours", start.sessionId))

        assertTrue(cancel.reply.contains("5 people on July 4, 2026 at 19:30, under Stephane"))
        assertEquals("RESERVATION_CANCELLATION", cancel.state.name)
        assertEquals("IDLE", confirmedCancel.state.name)
        assertEquals("opening_hours", openingHours.intent.wireName)
        assertTrue(openingHours.reply.contains("Monday"))
    }

    @Test
    fun `standalone conversation acts bypass NLP`() = runBlocking {
        val analyzer = ScriptedNlpAnalyzer {
            error("NLP should not be called for standalone conversation acts.")
        }
        val useCase = useCaseWithNlp(analyzer)

        val hello = useCase.handle(HandleChatMessageCommand("Hello", null))
        val thanks = useCase.handle(HandleChatMessageCommand("Thank you", hello.sessionId))
        val goodbye = useCase.handle(HandleChatMessageCommand("good bye", hello.sessionId))

        assertEquals("Hello. How can I help you today?", hello.reply)
        assertEquals(ConversationAct.GREETING, hello.conversationAct)
        assertEquals("You're welcome.", thanks.reply)
        assertEquals(ConversationAct.THANKS, thanks.conversationAct)
        assertEquals("Goodbye. See you soon.", goodbye.reply)
        assertEquals(ConversationAct.FAREWELL, goodbye.conversationAct)
        assertEquals(emptyList(), analyzer.texts)
    }

    @Test
    fun `contact request is a business intent`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer {
            analysis(IntentName.CONTACT_REQUEST)
        })

        val result = useCase.handle(HandleChatMessageCommand("What is your phone number?", null))

        assertEquals("contact_request", result.intent.wireName)
        assertEquals(null, result.conversationAct)
        assertTrue(result.reply.contains("+33 1 42 00 00 00"))
    }

    @Test
    fun `opening hours reply uses requested day from message`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer {
            analysis(IntentName.OPENING_HOURS)
        })

        val result = useCase.handle(
            HandleChatMessageCommand(
                "Will the restaurant still be serving dinner Sunday around tomorrow evening for a late booking?",
                null,
            )
        )

        assertEquals("opening_hours", result.intent.wireName)
        assertEquals("On Sunday, we are closed.", result.reply)
    }

    @Test
    fun `opening hours reply uses NLP date entity from context`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer {
            analysis(IntentName.OPENING_HOURS, entity(EntityType.DATE, "Sunday"))
        })

        val result = useCase.handle(HandleChatMessageCommand("Are you open then?", null))

        assertEquals("opening_hours", result.intent.wireName)
        assertEquals("On Sunday, we are closed.", result.reply)
    }

    @Test
    fun `menu replies use NLP menu intent`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer {
            analysis(IntentName.MENU_REQUEST)
        })

        val dessert = useCase.handle(HandleChatMessageCommand("Do you have some dessert?", null))
        val seafood = useCase.handle(HandleChatMessageCommand("Can I see the seafood options for weekend dinner?", dessert.sessionId))

        assertEquals("menu_request", dessert.intent.wireName)
        assertTrue(dessert.reply.contains("Chocolate Fondant"))
        assertEquals("menu_request", seafood.intent.wireName)
        assertTrue(seafood.reply.contains("Seared Sea Bass"))
    }

    @Test
    fun `reservation status intent does not reopen confirmation`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer { text, context ->
            when {
                "check my reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_STATUS)
                "reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_CREATE)
                text == "July 7" -> analysis(IntentName.UNKNOWN, entity(EntityType.DATE, text))
                text == "7pm" -> analysis(IntentName.UNKNOWN, entity(EntityType.TIME, text))
                text == "4" -> analysis(IntentName.UNKNOWN, entity(EntityType.PEOPLE_COUNT, text))
                else -> analysis(IntentName.UNKNOWN)
            }
        })
        val start = useCase.handle(HandleChatMessageCommand("I need a new reservation", null))
        useCase.handle(HandleChatMessageCommand("Thorng", start.sessionId))
        useCase.handle(HandleChatMessageCommand("July 7", start.sessionId))
        useCase.handle(HandleChatMessageCommand("7pm", start.sessionId))
        useCase.handle(HandleChatMessageCommand("4", start.sessionId))
        val confirmed = useCase.handle(HandleChatMessageCommand("yes you can confirm it", start.sessionId))

        val status = useCase.handle(HandleChatMessageCommand("check my reservation", start.sessionId))

        assertEquals("IDLE", confirmed.state.name)
        assertTrue(confirmed.completed)
        assertEquals("IDLE", status.state.name)
        assertTrue(status.reply.contains("Your reservation is confirmed: 4 people on July 7, 2026 at 19:00, under Thorng."))
    }

    @Test
    fun `unknown follow up keeps active modification workflow after informational switch`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer { text, _ ->
            when {
                "modify" in text.lowercase() -> analysis(IntentName.RESERVATION_MODIFY)
                "opening hours" in text.lowercase() -> analysis(IntentName.OPENING_HOURS)
                "reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_CREATE)
                text == "July 7" -> analysis(IntentName.UNKNOWN, entity(EntityType.DATE, text))
                text == "July 8" -> analysis(IntentName.UNKNOWN, entity(EntityType.DATE, text))
                text == "7pm" -> analysis(IntentName.UNKNOWN, entity(EntityType.TIME, text))
                text == "4" -> analysis(IntentName.UNKNOWN, entity(EntityType.PEOPLE_COUNT, text))
                else -> analysis(IntentName.UNKNOWN)
            }
        })
        val start = useCase.handle(HandleChatMessageCommand("I need a new reservation", null))
        useCase.handle(HandleChatMessageCommand("Thorng", start.sessionId))
        useCase.handle(HandleChatMessageCommand("July 7", start.sessionId))
        useCase.handle(HandleChatMessageCommand("7pm", start.sessionId))
        useCase.handle(HandleChatMessageCommand("4", start.sessionId))
        useCase.handle(HandleChatMessageCommand("yes", start.sessionId))

        val modify = useCase.handle(HandleChatMessageCommand("I want to modify my reservation", start.sessionId))
        val hours = useCase.handle(HandleChatMessageCommand("opening hours", start.sessionId))
        val newDate = useCase.handle(HandleChatMessageCommand("July 8", start.sessionId))

        assertEquals("reservation_modify", modify.intent.wireName)
        assertEquals("opening_hours", hours.intent.wireName)
        assertEquals("reservation_modify", newDate.intent.wireName)
        assertEquals("July 8, 2026", newDate.slots[SlotName.DATE])
        assertEquals("RESERVATION_MODIFICATION", newDate.state.name)
    }

    private fun useCaseWithNlp(nlpAnalyzer: NlpAnalyzer): HandleChatMessageUseCase {
        val knowledge = InMemoryRestaurantKnowledgeRepository()
        val inventory = InMemoryReservationInventoryRepository()
        val replies = ReplyComposer(knowledge)
        val clock = Clock.fixed(Instant.parse("2026-05-01T10:00:00Z"), ZoneOffset.UTC)
        val reservationWorkflowService = ReservationWorkflowService(inventory, replies, clock)
        val stateMachine = ChatStateMachine(
            idleStateHandler = IdleStateHandler(reservationWorkflowService, replies),
            workflowStateHandler = WorkflowStateHandler(reservationWorkflowService, replies),
        )
        val orchestrator = ChatMessageOrchestrator(
            nlpAnalyzer = nlpAnalyzer,
            conversationActPreprocessor = ConversationActPreprocessor(),
            intentResolver = IntentResolver(),
            stateMachine = stateMachine,
            replies = replies,
        )
        return HandleChatMessageUseCase(
            sessions = InMemoryConversationSessionRepository(),
            orchestrator = orchestrator,
            clock = clock,
        )
    }

    private class ScriptedNlpAnalyzer(
        private val script: (String, NlpAnalysisContext?) -> NlpAnalysis,
    ) : NlpAnalyzer {
        constructor(script: () -> NlpAnalysis) : this({ _, _ -> script() })

        val texts = mutableListOf<String>()

        override suspend fun analyze(text: String, domain: String, context: NlpAnalysisContext?): NlpAnalysis {
            texts.add(text)
            return script(text, context)
        }
    }

    private fun analysis(intentName: IntentName, vararg entities: NlpEntity): NlpAnalysis =
        NlpAnalysis(
            intent = NlpIntent(
                name = intentName,
                confidence = if (intentName == IntentName.UNKNOWN) 0.0 else 0.99,
                source = "test",
            ),
            entities = entities.toList(),
        )

    private fun entity(type: EntityType, value: String): NlpEntity =
        NlpEntity(
            type = type,
            value = value,
            confidence = 0.99,
            source = "test",
        )
}
