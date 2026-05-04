package dev.stephyu.core.chat

import dev.stephyu.core.chat.adapter.out.memory.InMemoryConversationSessionRepository
import dev.stephyu.core.chat.adapter.out.memory.InMemoryReservationInventoryRepository
import dev.stephyu.core.chat.adapter.out.memory.InMemoryRestaurantKnowledgeRepository
import dev.stephyu.core.chat.application.command.HandleConversationCommand
import dev.stephyu.core.chat.application.coordinator.ConversationCoordinator
import dev.stephyu.core.chat.application.intent.catalog.IntentCatalog
import dev.stephyu.core.chat.application.intent.decision.IntentDecisionEngine
import dev.stephyu.core.chat.application.intent.handler.IntentHandler
import dev.stephyu.core.chat.application.intent.handler.knowledge.*
import dev.stephyu.core.chat.application.intent.handler.reservation.ReservationCancelIntentHandler
import dev.stephyu.core.chat.application.intent.handler.reservation.ReservationCreateIntentHandler
import dev.stephyu.core.chat.application.intent.handler.reservation.ReservationModifyIntentHandler
import dev.stephyu.core.chat.application.intent.handler.reservation.ReservationStatusIntentHandler
import dev.stephyu.core.chat.application.port.out.NlpAnalyzer
import dev.stephyu.core.chat.application.signal.ConversationSignalExtractor
import dev.stephyu.core.chat.application.state.ConversationStateDispatcher
import dev.stephyu.core.chat.application.state.IdleStateHandler
import dev.stephyu.core.chat.application.state.WorkflowStateHandler
import dev.stephyu.core.chat.application.usecase.HandleConversationUseCase
import dev.stephyu.core.chat.application.workflow.*
import dev.stephyu.core.chat.domain.conversation.ConversationAct
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.nlp.*
import kotlinx.coroutines.runBlocking
import java.time.Clock
import java.time.Instant
import java.time.ZoneOffset
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class ChatOrchestrationTest {
    @Test
    fun `backend trusts NLP for reservation creation`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer {
            analysis(IntentName.RESERVATION_CREATE)
        })

        val result = useCase.handle(HandleConversationCommand("I want to make a reservation", null))

        assertEquals("reservation_create", result.intent.wireName)
        assertEquals("WORKFLOW", result.state.name)
    }

    @Test
    fun `leading greeting prefixes reply while NLP owns primary intent`() = runBlocking {
        val analyzer = ScriptedNlpAnalyzer {
            analysis(IntentName.RESERVATION_CREATE)
        }
        val useCase = useCaseWithNlp(analyzer)

        val result = useCase.handle(HandleConversationCommand("Hello, I want a reservation", null))

        assertEquals("reservation_create", result.intent.wireName)
        assertEquals(ConversationAct.GREETING, result.conversationAct)
        assertEquals("WORKFLOW", result.state.name)
        assertTrue(result.reply.startsWith("Hello. "))
        assertEquals(listOf("I want a reservation"), analyzer.texts)
    }

    @Test
    fun `low signal idle message does not trust random high confidence NLP intent`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer {
            analysis(
                intentName = IntentName.RESERVATION_CANCEL,
                utteranceKind = NlpUtteranceKind.UNKNOWN,
            )
        })

        val result = useCase.handle(HandleConversationCommand("j", null))

        assertEquals("unknown", result.intent.wireName)
        assertEquals("IDLE", result.state.name)
    }

    @Test
    fun `idle yes does not trigger random intent`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer {
            analysis(
                intentName = IntentName.MENU_REQUEST,
                utteranceKind = NlpUtteranceKind.UNKNOWN,
            )
        })

        val result = useCase.handle(HandleConversationCommand("yes", null))

        assertEquals("unknown", result.intent.wireName)
        assertEquals("IDLE", result.state.name)
    }

    @Test
    fun `idle ambiguous standalone token does not trigger random intent`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer {
            analysis(
                intentName = IntentName.RESERVATION_STATUS,
                utteranceKind = NlpUtteranceKind.UNKNOWN,
            )
        })

        val result = useCase.handle(HandleConversationCommand("miss", null))

        assertEquals("unknown", result.intent.wireName)
        assertEquals("IDLE", result.state.name)
    }

    @Test
    fun `idle personal small talk does not trigger random business intent`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer {
            analysis(
                intentName = IntentName.PRICING_REQUEST,
                utteranceKind = NlpUtteranceKind.SMALL_TALK,
            )
        })

        val result = useCase.handle(HandleConversationCommand("how am I", null))

        assertEquals("unknown", result.intent.wireName)
        assertEquals("IDLE", result.state.name)
    }

    @Test
    fun `unsupported standalone content falls back to unknown`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer {
            analysis(
                intentName = IntentName.MENU_REQUEST,
                confidence = 0.94,
                alternatives = mapOf(IntentName.PRICING_REQUEST to 0.03),
                utteranceKind = NlpUtteranceKind.OUT_OF_DOMAIN,
            )
        })

        val result = useCase.handle(HandleConversationCommand("carrot", null))

        assertEquals("unknown", result.intent.wireName)
        assertEquals("IDLE", result.state.name)
        assertTrue(result.reply.contains("I did not understand"))
    }

    @Test
    fun `vague follow up without topic falls back to unknown`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer {
            analysis(
                intentName = IntentName.PRICING_REQUEST,
                confidence = 0.72,
                alternatives = mapOf(IntentName.MENU_REQUEST to 0.24),
                utteranceKind = NlpUtteranceKind.VAGUE_FOLLOW_UP,
            )
        })

        val result = useCase.handle(HandleConversationCommand("what else ?", null))

        assertEquals("unknown", result.intent.wireName)
        assertEquals("IDLE", result.state.name)
    }

    @Test
    fun `vague follow up reuses last informational topic`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer { text, _ ->
            when {
                "dessert" in text.lowercase() -> analysis(IntentName.MENU_REQUEST)
                "what else" in text.lowercase() -> analysis(
                    intentName = IntentName.PRICING_REQUEST,
                    confidence = 0.51,
                    alternatives = mapOf(IntentName.MENU_REQUEST to 0.46),
                    utteranceKind = NlpUtteranceKind.VAGUE_FOLLOW_UP,
                )

                else -> analysis(IntentName.UNKNOWN)
            }
        })

        val menu = useCase.handle(HandleConversationCommand("Do you have some dessert?", null))
        val followUp = useCase.handle(HandleConversationCommand("what else?", menu.sessionId))

        assertEquals("menu_request", menu.intent.wireName)
        assertEquals("menu_request", followUp.intent.wireName)
        assertTrue(followUp.reply.contains("Menu highlights"))
    }

    @Test
    fun `NLP vague follow up utterance reuses last informational topic`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer { text, _ ->
            when {
                "dessert" in text.lowercase() -> analysis(IntentName.MENU_REQUEST)
                "more" in text.lowercase() -> analysis(
                    intentName = IntentName.PRICING_REQUEST,
                    confidence = 0.9,
                    utteranceKind = NlpUtteranceKind.VAGUE_FOLLOW_UP,
                )

                else -> analysis(IntentName.UNKNOWN)
            }
        })

        val menu = useCase.handle(HandleConversationCommand("Do you have some dessert?", null))
        val followUp = useCase.handle(HandleConversationCommand("more?", menu.sessionId))

        assertEquals("menu_request", menu.intent.wireName)
        assertEquals("menu_request", followUp.intent.wireName)
        assertTrue(followUp.reply.contains("Menu highlights"))
    }

    @Test
    fun `NLP non business utterance prevents random business routing`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer {
            analysis(
                intentName = IntentName.PRICING_REQUEST,
                confidence = 0.94,
                utteranceKind = NlpUtteranceKind.SMALL_TALK,
            )
        })

        val result = useCase.handle(HandleConversationCommand("how are you?", null))

        assertEquals("unknown", result.intent.wireName)
        assertEquals("IDLE", result.state.name)
    }

    @Test
    fun `missing NLP utterance falls back to safe unknown`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer {
            NlpAnalysis(
                intent = NlpIntent(
                    name = IntentName.MENU_REQUEST,
                    confidence = 0.99,
                    source = "legacy_test",
                ),
            )
        })

        val result = useCase.handle(HandleConversationCommand("old contract response", null))

        assertEquals("unknown", result.intent.wireName)
        assertEquals("IDLE", result.state.name)
    }

    @Test
    fun `ambiguous utterance with low margin informational intents triggers clarification`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer {
            analysis(
                intentName = IntentName.MENU_REQUEST,
                confidence = 0.52,
                alternatives = mapOf(IntentName.PRICING_REQUEST to 0.45),
                utteranceKind = NlpUtteranceKind.AMBIGUOUS,
            )
        })

        val result = useCase.handle(HandleConversationCommand("tell me more about dinner", null))

        assertEquals("unknown", result.intent.wireName)
        assertEquals("IDLE", result.state.name)
        assertTrue(result.reply.contains("menu options"))
        assertTrue(result.reply.contains("pricing information"))
    }

    @Test
    fun `clarification reply can resolve to chosen candidate`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer { text, _ ->
            when {
                "dinner" in text.lowercase() -> analysis(
                    intentName = IntentName.MENU_REQUEST,
                    confidence = 0.52,
                    alternatives = mapOf(IntentName.PRICING_REQUEST to 0.45),
                )

                "pricing" in text.lowercase() -> analysis(IntentName.PRICING_REQUEST)
                else -> analysis(IntentName.UNKNOWN)
            }
        })

        val first = useCase.handle(HandleConversationCommand("tell me more about dinner", null))
        val second = useCase.handle(HandleConversationCommand("pricing", first.sessionId))

        assertEquals("unknown", first.intent.wireName)
        assertEquals("pricing_request", second.intent.wireName)
        assertTrue(second.reply.contains("Price guide"))
    }

    @Test
    fun `active reservation flow continues on unknown NLP intent`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer { text, context ->
            when {
                "reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_CREATE)
                context?.requiredSlots?.contains(SlotName.DATE) == true && "July 4" in text ->
                    analysis(IntentName.UNKNOWN, entity(SlotName.DATE, "July 4"))

                context?.requiredSlots?.contains(SlotName.TIME) == true && "7:30pm" in text ->
                    analysis(IntentName.UNKNOWN, entity(SlotName.TIME, "7:30pm"))

                context?.requiredSlots?.contains(SlotName.PEOPLE) == true && text == "5" ->
                    analysis(IntentName.UNKNOWN, entity(SlotName.PEOPLE, "5"))

                else -> analysis(IntentName.UNKNOWN)
            }
        })

        val start = useCase.handle(HandleConversationCommand("I need a new reservation", null))
        val name = useCase.handle(HandleConversationCommand("Stephane", start.sessionId))
        val date = useCase.handle(HandleConversationCommand("July 4", start.sessionId))
        val time = useCase.handle(HandleConversationCommand("7:30pm", start.sessionId))
        val people = useCase.handle(HandleConversationCommand("5", start.sessionId))

        assertEquals("WORKFLOW", name.state.name)
        assertEquals("July 4, 2026", date.slots[SlotName.DATE])
        assertEquals("19:30", time.slots[SlotName.TIME])
        assertEquals("WORKFLOW", people.state.name)
        assertTrue(people.reply.contains("at 19:30"))
    }

    @Test
    fun `canonical NLP entity values fill workflow slots`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer { text, context ->
            when {
                "reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_CREATE)
                context?.requiredSlots?.contains(SlotName.DATE) == true && "tomorrow" in text.lowercase() ->
                    analysis(
                        IntentName.UNKNOWN,
                        entity(SlotName.DATE, value = "2026-05-02", rawValue = "tomorrow"),
                        entity(SlotName.TIME, value = "19:00", rawValue = "7pm"),
                        entity(SlotName.PEOPLE, value = "5", rawValue = "for 5 ppl"),
                    )

                else -> analysis(IntentName.UNKNOWN)
            }
        })

        val start = useCase.handle(HandleConversationCommand("I need a new reservation", null))
        useCase.handle(HandleConversationCommand("Stephane", start.sessionId))
        val result = useCase.handle(HandleConversationCommand("tomorrow at 7pm for 5 ppl", start.sessionId))

        assertEquals("May 2, 2026", result.slots[SlotName.DATE])
        assertEquals("19:00", result.slots[SlotName.TIME])
        assertEquals("5", result.slots[SlotName.PEOPLE])
        assertEquals("WORKFLOW", result.state.name)
        assertTrue(result.reply.contains("Should I confirm it?"))
    }


    @Test
    fun `blank message preserves active workflow context`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer { text, context ->
            when {
                "reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_CREATE)
                context?.requiredSlots?.contains(SlotName.DATE) == true && "July 4" in text ->
                    analysis(IntentName.UNKNOWN, entity(SlotName.DATE, "July 4"))

                else -> analysis(IntentName.UNKNOWN)
            }
        })

        val start = useCase.handle(HandleConversationCommand("I need a new reservation", null))
        val name = useCase.handle(HandleConversationCommand("Stephane", start.sessionId))
        val blank = useCase.handle(HandleConversationCommand("   ", start.sessionId))

        assertEquals("WORKFLOW", name.state.name)
        assertEquals("WORKFLOW", blank.state.name)
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
                text == "July 4" -> analysis(IntentName.UNKNOWN, entity(SlotName.DATE, text))
                text == "7:30pm" -> analysis(IntentName.UNKNOWN, entity(SlotName.TIME, text))
                text == "5" -> analysis(IntentName.UNKNOWN, entity(SlotName.PEOPLE, text))
                else -> analysis(IntentName.UNKNOWN)
            }
        })

        val start = useCase.handle(HandleConversationCommand("I need a new reservation", null))
        useCase.handle(HandleConversationCommand("Stephane", start.sessionId))
        useCase.handle(HandleConversationCommand("July 4", start.sessionId))
        useCase.handle(HandleConversationCommand("7:30pm", start.sessionId))
        useCase.handle(HandleConversationCommand("5", start.sessionId))
        useCase.handle(HandleConversationCommand("yes", start.sessionId))

        val cancel = useCase.handle(HandleConversationCommand("I want to cancel my reservation", start.sessionId))
        val confirmedCancel = useCase.handle(HandleConversationCommand("yes", start.sessionId))
        val openingHours = useCase.handle(HandleConversationCommand("opening hours", start.sessionId))

        assertTrue(cancel.reply.contains("5 people on July 4, 2026 at 19:30, under Stephane"))
        assertEquals("WORKFLOW", cancel.state.name)
        assertEquals("IDLE", confirmedCancel.state.name)
        assertEquals("opening_hours", openingHours.intent.wireName)
        assertTrue(openingHours.reply.contains("Monday"))
    }

    @Test
    fun `informational detour does not fill later slots before first missing one`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer { text, _ ->
            when {
                "reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_CREATE)
                "open sunday" in text.lowercase() -> analysis(
                    IntentName.OPENING_HOURS,
                    entity(SlotName.DATE, "Sunday"),
                )

                else -> analysis(IntentName.UNKNOWN)
            }
        })

        val start = useCase.handle(HandleConversationCommand("I want book a reservation", null))
        val hours = useCase.handle(HandleConversationCommand("Are you open Sunday ?", start.sessionId))

        assertEquals("opening_hours", hours.intent.wireName)
        assertEquals(listOf(SlotName.NAME, SlotName.DATE, SlotName.TIME, SlotName.PEOPLE), hours.missingSlots)
        assertTrue(hours.reply.contains("What name should I use for the reservation?"))
    }

    @Test
    fun `active reservation creation cancel aborts workflow without NLP`() = runBlocking {
        val analyzer = ScriptedNlpAnalyzer { text, context ->
            when {
                "reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_CREATE)
                context?.requiredSlots?.contains(SlotName.DATE) == true && "July 4" in text ->
                    analysis(IntentName.UNKNOWN, entity(SlotName.DATE, "July 4"))

                else -> analysis(IntentName.UNKNOWN)
            }
        }
        val useCase = useCaseWithNlp(analyzer)

        val start = useCase.handle(HandleConversationCommand("I need a new reservation", null))
        useCase.handle(HandleConversationCommand("Stephane", start.sessionId))
        val cancel = useCase.handle(HandleConversationCommand("cancel", start.sessionId))

        assertEquals("IDLE", cancel.state.name)
        assertTrue(cancel.reply.contains("I have cancelled the current reservation request"))
        assertEquals(listOf("I need a new reservation", "Stephane"), analyzer.texts)
    }

    @Test
    fun `active reservation modify stop aborts workflow`() = runBlocking {
        val analyzer = ScriptedNlpAnalyzer { text, _ ->
            when {
                "modify" in text.lowercase() -> analysis(IntentName.RESERVATION_MODIFY)
                "reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_CREATE)
                text == "July 7" -> analysis(IntentName.UNKNOWN, entity(SlotName.DATE, text))
                text == "7pm" -> analysis(IntentName.UNKNOWN, entity(SlotName.TIME, text))
                text == "4" -> analysis(IntentName.UNKNOWN, entity(SlotName.PEOPLE, text))
                else -> analysis(IntentName.UNKNOWN)
            }
        }
        val useCase = useCaseWithNlp(analyzer)

        val start = useCase.handle(HandleConversationCommand("I need a new reservation", null))
        useCase.handle(HandleConversationCommand("Thorng", start.sessionId))
        useCase.handle(HandleConversationCommand("July 7", start.sessionId))
        useCase.handle(HandleConversationCommand("7pm", start.sessionId))
        useCase.handle(HandleConversationCommand("4", start.sessionId))
        useCase.handle(HandleConversationCommand("yes", start.sessionId))
        useCase.handle(HandleConversationCommand("I want to modify my reservation", start.sessionId))
        val stop = useCase.handle(HandleConversationCommand("stop", start.sessionId))

        assertEquals("IDLE", stop.state.name)
        assertTrue(stop.reply.contains("I have cancelled the current reservation update request"))
    }

    @Test
    fun `non cancellable workflow ignores cancel command and stays active`() = runBlocking {
        val analyzer = ScriptedNlpAnalyzer { text, _ ->
            when {
                "cancel my reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_CANCEL)
                "reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_CREATE)
                text == "July 7" -> analysis(IntentName.UNKNOWN, entity(SlotName.DATE, text))
                text == "7pm" -> analysis(IntentName.UNKNOWN, entity(SlotName.TIME, text))
                text == "4" -> analysis(IntentName.UNKNOWN, entity(SlotName.PEOPLE, text))
                else -> analysis(IntentName.UNKNOWN)
            }
        }
        val useCase = useCaseWithNlp(analyzer)

        val start = useCase.handle(HandleConversationCommand("I need a new reservation", null))
        useCase.handle(HandleConversationCommand("Thorng", start.sessionId))
        useCase.handle(HandleConversationCommand("July 7", start.sessionId))
        useCase.handle(HandleConversationCommand("7pm", start.sessionId))
        useCase.handle(HandleConversationCommand("4", start.sessionId))
        useCase.handle(HandleConversationCommand("yes", start.sessionId))
        val cancelReservation = useCase.handle(HandleConversationCommand("cancel my reservation", start.sessionId))
        val cancelCommand = useCase.handle(HandleConversationCommand("cancel", start.sessionId))

        assertEquals("WORKFLOW", cancelReservation.state.name)
        assertEquals("WORKFLOW", cancelCommand.state.name)
        assertTrue(cancelCommand.reply.contains("Should I cancel it?"))
    }

    @Test
    fun `invalid party size keeps active workflow even if NLP misclassifies intent`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer { text, context ->
            when {
                "reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_CREATE)
                context?.requiredSlots?.contains(SlotName.DATE) == true && "July 7" in text ->
                    analysis(IntentName.UNKNOWN, entity(SlotName.DATE, "July 7"))

                context?.requiredSlots?.contains(SlotName.TIME) == true && "7pm" in text ->
                    analysis(IntentName.UNKNOWN, entity(SlotName.TIME, "7pm"))

                text == "100" -> analysis(IntentName.RESERVATION_STATUS)
                else -> analysis(IntentName.UNKNOWN)
            }
        })

        val start = useCase.handle(HandleConversationCommand("I need a new reservation", null))
        useCase.handle(HandleConversationCommand("Thorng", start.sessionId))
        useCase.handle(HandleConversationCommand("July 7", start.sessionId))
        useCase.handle(HandleConversationCommand("7pm", start.sessionId))
        val invalidPeople = useCase.handle(HandleConversationCommand("100", start.sessionId))

        assertEquals("reservation_create", invalidPeople.intent.wireName)
        assertEquals("WORKFLOW", invalidPeople.state.name)
        assertEquals(listOf(SlotName.PEOPLE), invalidPeople.missingSlots)
        assertTrue(invalidPeople.reply.contains("We can accept parties from 1 to 12 people"))
    }

    @Test
    fun `invalid late reservation time keeps workflow active`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer { text, context ->
            when {
                "reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_CREATE)
                context?.requiredSlots?.contains(SlotName.DATE) == true && "July 7" in text ->
                    analysis(IntentName.UNKNOWN, entity(SlotName.DATE, "July 7"))

                context?.requiredSlots?.contains(SlotName.TIME) == true && "11:59pm" in text ->
                    analysis(IntentName.UNKNOWN, entity(SlotName.TIME, "11:59pm"))

                else -> analysis(IntentName.UNKNOWN)
            }
        })

        val start = useCase.handle(HandleConversationCommand("I need a new reservation", null))
        useCase.handle(HandleConversationCommand("Thorng", start.sessionId))
        useCase.handle(HandleConversationCommand("July 7", start.sessionId))
        val invalidTime = useCase.handle(HandleConversationCommand("at 11:59pm", start.sessionId))

        assertEquals("WORKFLOW", invalidTime.state.name)
        assertEquals(listOf(SlotName.TIME, SlotName.PEOPLE), invalidTime.missingSlots)
        assertTrue(invalidTime.reply.contains("between 11:30 and 23:30"))
    }

    @Test
    fun `workflow confirmation reply reports workflow intent even if NLP misclassifies`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer { text, context ->
            when {
                "reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_CREATE)
                context?.requiredSlots?.contains(SlotName.DATE) == true && "July 9" in text ->
                    analysis(IntentName.UNKNOWN, entity(SlotName.DATE, "July 9"))

                context?.requiredSlots?.contains(SlotName.TIME) == true && "11pm" in text ->
                    analysis(IntentName.UNKNOWN, entity(SlotName.TIME, "11pm"))

                text.contains("Stephane", ignoreCase = true) && text.contains("3", ignoreCase = false) ->
                    analysis(
                        IntentName.RESERVATION_CANCEL,
                        entity(SlotName.NAME, "Stephane"),
                        entity(SlotName.PEOPLE, "3")
                    )

                text == "yes" -> analysis(IntentName.MENU_REQUEST)
                else -> analysis(IntentName.UNKNOWN)
            }
        })

        val start = useCase.handle(HandleConversationCommand("I want book a reservation", null))
        useCase.handle(HandleConversationCommand("July 9", start.sessionId))
        useCase.handle(HandleConversationCommand("11pm", start.sessionId))
        val confirmation = useCase.handle(HandleConversationCommand("Stephane and for 3 people", start.sessionId))
        val confirmed = useCase.handle(HandleConversationCommand("yes", start.sessionId))

        assertEquals("reservation_create", confirmation.intent.wireName)
        assertEquals("WORKFLOW", confirmation.state.name)
        assertTrue(confirmation.reply.contains("Should I confirm it?"))
        assertEquals("reservation_create", confirmed.intent.wireName)
        assertEquals("IDLE", confirmed.state.name)
        assertTrue(confirmed.completed)
    }

    @Test
    fun `cancel my reservation without active workflow still routes to reservation cancel intent`() = runBlocking {
        val analyzer = ScriptedNlpAnalyzer { text, _ ->
            when {
                "cancel my reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_CANCEL)
                "reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_CREATE)
                text == "July 7" -> analysis(IntentName.UNKNOWN, entity(SlotName.DATE, text))
                text == "7pm" -> analysis(IntentName.UNKNOWN, entity(SlotName.TIME, text))
                text == "4" -> analysis(IntentName.UNKNOWN, entity(SlotName.PEOPLE, text))
                else -> analysis(IntentName.UNKNOWN)
            }
        }
        val useCase = useCaseWithNlp(analyzer)

        val start = useCase.handle(HandleConversationCommand("I need a new reservation", null))
        useCase.handle(HandleConversationCommand("Thorng", start.sessionId))
        useCase.handle(HandleConversationCommand("July 7", start.sessionId))
        useCase.handle(HandleConversationCommand("7pm", start.sessionId))
        useCase.handle(HandleConversationCommand("4", start.sessionId))
        useCase.handle(HandleConversationCommand("yes", start.sessionId))
        val cancelReservation = useCase.handle(HandleConversationCommand("cancel my reservation", start.sessionId))

        assertEquals("reservation_cancel", cancelReservation.intent.wireName)
        assertEquals("WORKFLOW", cancelReservation.state.name)
        assertTrue(analyzer.texts.contains("cancel my reservation"))
    }

    @Test
    fun `standalone conversation acts bypass NLP`() = runBlocking {
        val analyzer = ScriptedNlpAnalyzer {
            error("NLP should not be called for standalone conversation acts.")
        }
        val useCase = useCaseWithNlp(analyzer)

        val hello = useCase.handle(HandleConversationCommand("Hello", null))
        val thanks = useCase.handle(HandleConversationCommand("Thank you", hello.sessionId))
        val goodbye = useCase.handle(HandleConversationCommand("good bye", hello.sessionId))

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

        val result = useCase.handle(HandleConversationCommand("What is your phone number?", null))

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
            HandleConversationCommand(
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
            analysis(IntentName.OPENING_HOURS, entity(SlotName.DATE, "Sunday"))
        })

        val result = useCase.handle(HandleConversationCommand("Are you open then?", null))

        assertEquals("opening_hours", result.intent.wireName)
        assertEquals("On Sunday, we are closed.", result.reply)
    }

    @Test
    fun `menu replies use NLP menu intent`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer {
            analysis(IntentName.MENU_REQUEST)
        })

        val dessert = useCase.handle(HandleConversationCommand("Do you have some dessert?", null))
        val seafood = useCase.handle(
            HandleConversationCommand(
                "Can I see the seafood options for weekend dinner?",
                dessert.sessionId
            )
        )

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
                text == "July 7" -> analysis(IntentName.UNKNOWN, entity(SlotName.DATE, text))
                text == "7pm" -> analysis(IntentName.UNKNOWN, entity(SlotName.TIME, text))
                text == "4" -> analysis(IntentName.UNKNOWN, entity(SlotName.PEOPLE, text))
                else -> analysis(IntentName.UNKNOWN)
            }
        })
        val start = useCase.handle(HandleConversationCommand("I need a new reservation", null))
        useCase.handle(HandleConversationCommand("Thorng", start.sessionId))
        useCase.handle(HandleConversationCommand("July 7", start.sessionId))
        useCase.handle(HandleConversationCommand("7pm", start.sessionId))
        useCase.handle(HandleConversationCommand("4", start.sessionId))
        val confirmed = useCase.handle(HandleConversationCommand("yes you can confirm it", start.sessionId))

        val status = useCase.handle(HandleConversationCommand("check my reservation", start.sessionId))

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
                text == "July 7" -> analysis(IntentName.UNKNOWN, entity(SlotName.DATE, text))
                text == "July 8" -> analysis(IntentName.UNKNOWN, entity(SlotName.DATE, text))
                text == "7pm" -> analysis(IntentName.UNKNOWN, entity(SlotName.TIME, text))
                text == "4" -> analysis(IntentName.UNKNOWN, entity(SlotName.PEOPLE, text))
                else -> analysis(IntentName.UNKNOWN)
            }
        })
        val start = useCase.handle(HandleConversationCommand("I need a new reservation", null))
        useCase.handle(HandleConversationCommand("Thorng", start.sessionId))
        useCase.handle(HandleConversationCommand("July 7", start.sessionId))
        useCase.handle(HandleConversationCommand("7pm", start.sessionId))
        useCase.handle(HandleConversationCommand("4", start.sessionId))
        useCase.handle(HandleConversationCommand("yes", start.sessionId))

        val modify = useCase.handle(HandleConversationCommand("I want to modify my reservation", start.sessionId))
        val hours = useCase.handle(HandleConversationCommand("opening hours", start.sessionId))
        val newDate = useCase.handle(HandleConversationCommand("July 8", start.sessionId))

        assertEquals("reservation_modify", modify.intent.wireName)
        assertEquals("opening_hours", hours.intent.wireName)
        assertEquals("reservation_modify", newDate.intent.wireName)
        assertEquals("July 8, 2026", newDate.slots[SlotName.DATE])
        assertEquals("WORKFLOW", newDate.state.name)
    }

    @Test
    fun `informational detour enriches active workflow in background`() = runBlocking {
        val useCase = useCaseWithNlp(ScriptedNlpAnalyzer { text, context ->
            when {
                "reservation" in text.lowercase() -> analysis(IntentName.RESERVATION_CREATE)
                "open" in text.lowercase() -> analysis(
                    IntentName.OPENING_HOURS,
                    entity(SlotName.DATE, "Friday"),
                    entity(SlotName.TIME, "8pm"),
                )

                context?.requiredSlots?.contains(SlotName.PEOPLE) == true && text == "4" ->
                    analysis(IntentName.UNKNOWN, entity(SlotName.PEOPLE, "4"))

                else -> analysis(IntentName.UNKNOWN)
            }
        })

        val start = useCase.handle(HandleConversationCommand("I need a new reservation", null))
        useCase.handle(HandleConversationCommand("Stephane", start.sessionId))
        val hours = useCase.handle(HandleConversationCommand("are you open friday at 8pm?", start.sessionId))

        assertEquals("opening_hours", hours.intent.wireName)
        assertEquals("May 8, 2026", hours.slots[SlotName.DATE])
        assertEquals(null, hours.slots[SlotName.TIME])
        assertEquals(listOf(SlotName.TIME, SlotName.PEOPLE), hours.missingSlots)
        assertTrue(hours.reply.contains("Friday"))
        assertTrue(hours.reply.contains("Next: What time would you like?"))
    }

    private fun useCaseWithNlp(nlpAnalyzer: NlpAnalyzer): HandleConversationUseCase {
        val knowledge = InMemoryRestaurantKnowledgeRepository()
        val inventory = InMemoryReservationInventoryRepository()
        val clock = Clock.fixed(Instant.parse("2026-05-01T10:00:00Z"), ZoneOffset.UTC)
        val workflowRules: List<WorkflowStateRule> = listOf(
            CancelWorkflowRule(),
            FillRequirementsRule(),
            EnterConfirmationRule(),
            ResolveConfirmationRule(),
        )
        val workflowEngine = WorkflowEngine(workflowRules, clock)
        val intentServices: List<IntentHandler> = listOf(
            ReservationCreateIntentHandler(workflowEngine, inventory, clock),
            ReservationModifyIntentHandler(workflowEngine, inventory, clock),
            ReservationCancelIntentHandler(workflowEngine),
            ReservationStatusIntentHandler(),
            MenuRequestIntentHandler(knowledge),
            OpeningHoursIntentHandler(knowledge),
            LocationRequestIntentHandler(knowledge),
            PricingRequestIntentHandler(knowledge),
            ContactRequestIntentHandler(knowledge),
        )
        val conversationConfig = IntentCatalog(
            intentServices = intentServices,
        )
        val stateMachine = ConversationStateDispatcher(
            idleStateHandler = IdleStateHandler(conversationConfig),
            workflowStateHandler = WorkflowStateHandler(conversationConfig),
        )
        val orchestrator = ConversationCoordinator(
            nlpAnalyzer = nlpAnalyzer,
            signalExtractor = ConversationSignalExtractor(),
            intentDecisionEngine = IntentDecisionEngine(conversationConfig),
            stateDispatcher = stateMachine,
        )
        return HandleConversationUseCase(
            conversationSessionRepository = InMemoryConversationSessionRepository(),
            coordinator = orchestrator,
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

    private fun analysis(
        intentName: IntentName,
        vararg entities: NlpEntity,
        confidence: Double = if (intentName == IntentName.UNKNOWN) 0.0 else 0.99,
        alternatives: Map<IntentName, Double> = emptyMap(),
        utteranceKind: NlpUtteranceKind = NlpUtteranceKind.BUSINESS_QUERY,
    ): NlpAnalysis =
        NlpAnalysis(
            intent = NlpIntent(
                name = intentName,
                confidence = confidence,
                source = "test",
                alternatives = alternatives,
            ),
            entities = entities.toList(),
            utterance = NlpUtterance(
                kind = utteranceKind,
                confidence = if (utteranceKind == NlpUtteranceKind.BUSINESS_QUERY) 1.0 else 0.9,
                source = "test",
            ),
        )

    private fun entity(type: SlotName, value: String, rawValue: String = value): NlpEntity =
        NlpEntity(
            type = type,
            value = value,
            confidence = 0.99,
            source = "test",
            rawValue = rawValue,
        )
}

