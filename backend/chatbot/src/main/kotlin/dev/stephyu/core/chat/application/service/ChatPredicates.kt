package dev.stephyu.core.chat.application.service

import dev.stephyu.core.chat.domain.ConversationSession
import dev.stephyu.core.chat.domain.IntentName

fun IntentName.isInformationalIntent(): Boolean =
    this in setOf(
        IntentName.MENU_REQUEST,
        IntentName.OPENING_HOURS,
        IntentName.LOCATION_REQUEST,
        IntentName.PRICING_REQUEST,
        IntentName.CONTACT_REQUEST,
    )

fun IntentName.isReservationWorkflowIntent(): Boolean =
    this in setOf(
        IntentName.RESERVATION_CREATE,
        IntentName.RESERVATION_MODIFY,
        IntentName.RESERVATION_CANCEL,
    )

fun ConversationSession.hasActiveReservationWorkflow(): Boolean =
    currentWorkflow != null
